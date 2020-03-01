use std::convert::From;
use std::mem;
use std::str::FromStr;

use gio::prelude::*;
use gio::{
    IOStream, IOStreamExt, SocketClient, SocketClientExt, SocketConnection, TlsCertificateFlags,
};
use glib::source::PRIORITY_DEFAULT;
use glib::Cast;
use lazy_static::lazy_static;
use relm::{connect_async, Relm, Update, UpdateNew};
use serde_yaml;

use regex::Regex;
use url::Url;

use super::super::errors::{RustamanError, RustamanResult};
use super::super::models::Environment;
use super::handlebars::compile_template;

const READ_SIZE: usize = 1024;
lazy_static! {
    pub static ref RE_EXTRACT_AUTHORITY_FROM_DIRECTIVE: Regex =
        Regex::new(r"#![\s]*Authority:[\s]*(?P<host>.+):(?P<port>[0-9]+)").unwrap();
    pub static ref RE_EXTRACT_INSECURE_FLAG: Regex =
        Regex::new(r"#![\s]*AllowInsecureCertificate").unwrap();
    pub static ref RE_SPLIT_HTTP_FIRST_LINE: Regex = Regex::new("[ ]+").unwrap();
    pub static ref RE_EXTRACT_CAPTURE: Regex =
        Regex::new(r"#![\s]*Capture:\s*(?P<capture>.+)").unwrap();
    pub static ref RE_SPLIT_END_CAPTURE: Regex = Regex::new(r"#![\s]*EndCapture").unwrap();
}

fn parse_response(response: &str) -> Option<serde_yaml::Value> {
    let mut is_json = false;
    let mut has_content = true;
    let mut text = String::new();
    let mut lines = response.lines();
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                if unwrapped.is_empty() {
                    break;
                }
                if unwrapped.starts_with("Content-Type: application/json") {
                    is_json = true;
                }
            }
            None => has_content = false,
        }
    }
    if has_content {
        loop {
            let line = lines.next();
            match line {
                Some(unwrapped) => {
                    text.push_str(unwrapped);
                    text.push('\n');
                }
                None => break,
            }
        }
    };
    if is_json && has_content {
        let resp: serde_yaml::Value = serde_json::from_str(text.as_str()).unwrap();
        Some(resp)
    } else {
        None
    }
}

fn extract_authority_from_directive(line: &str) -> Option<(String, u16)> {
    let resp = RE_EXTRACT_AUTHORITY_FROM_DIRECTIVE
        .captures(line)
        .and_then(|cap| {
            let host = cap
                .name("host")
                .map(|host| host.as_str().trim_start_matches("[").trim_end_matches("]"));
            let port = cap
                .name("port")
                .map(|port| FromStr::from_str(port.as_str()).unwrap());
            Some((host.unwrap().to_string(), port.unwrap()))
        });
    resp
}

fn extract_capture_name(line: &str) -> Option<String> {
    let resp = RE_EXTRACT_CAPTURE.captures(line).and_then(|cap| {
        let cap = cap.name("capture");
        if let Some(capture) = cap {
            Some(capture.as_str().to_string())
        } else {
            None
        }
    });
    resp
}

fn extract_insecure_flag(line: &str) -> bool {
    return RE_EXTRACT_INSECURE_FLAG.is_match(line);
}

#[derive(Debug, PartialEq, Clone)]
pub enum Scheme {
    HTTP,
    HTTPS,
    Err(String), // until TryFrom is stable
}

impl<'a> From<&'a str> for Scheme {
    fn from(value: &str) -> Self {
        match value {
            "http" => Scheme::HTTP,
            "https" => Scheme::HTTPS,
            _ => Scheme::Err(value.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    scheme: Scheme,
    host: String,
    port: u16,
    http_frame: String,
    tls_flags: TlsCertificateFlags,
    capture: Option<String>,
}

impl HttpRequest {
    pub fn http_frame(&self) -> &str {
        self.http_frame.as_str()
    }

    pub fn authority(&self) -> (&str, u16) {
        (self.host.as_str(), self.port)
    }

    /// Obfusface the http_frame
    pub fn obfuscate(&self, env: &Environment) -> HttpRequest {
        let mut req = self.clone();
        let s = env.obfuscated_string();
        let _: Vec<_> = s
            .iter()
            .map(|ref x| {
                let obf = format!("{}...", &x[0..3]);
                req.http_frame = req.http_frame.replace(x.as_str(), obf.as_str())
            })
            .collect();
        req
    }
}

pub fn parse_request(request: &str) -> RustamanResult<HttpRequest> {
    info!("Parsing request {}", request.len());

    let mut lines = request.lines();
    let mut line = lines.next();
    let mut authority: Option<(String, u16)> = None;
    let mut tls_flags = TlsCertificateFlags::all();
    let mut capture = None;

    loop {
        if line.is_none() {
            break;
        }
        let unwrapped = line.unwrap();
        if !unwrapped.is_empty() && !unwrapped.starts_with('#') {
            break;
        }
        if let Some(auth) = extract_authority_from_directive(unwrapped) {
            debug!("Authority found from the request comment: {:?}", auth);
            authority = Some(auth);
        } else if let Some(cap) = extract_capture_name(unwrapped) {
            capture = Some(cap);
        } else if extract_insecure_flag(unwrapped) {
            tls_flags = TlsCertificateFlags::empty();
        } else {
            debug!("Ignoring comment {}", unwrapped);
        }
        line = lines.next();
    }
    if line.is_none() {
        error!("No request found");
        return Err(RustamanError::RequestParsingError(
            "No request found".to_owned(),
        ));
    }

    info!("Parsing First line {:?}", line);
    let verb_url_version: Vec<&str> = RE_SPLIT_HTTP_FIRST_LINE.split(line.unwrap()).collect();
    let (verb, url, version) = match verb_url_version.len() {
        2 => (verb_url_version[0], verb_url_version[1], "HTTP/1.1"),
        3 => (
            verb_url_version[0],
            verb_url_version[1],
            verb_url_version[2],
        ),
        _ => {
            error!("Parse error on line: {}", line.unwrap());
            return Err(RustamanError::RequestParsingError(
                format!("Parse error on line: {}", line.unwrap()).to_owned(),
            ));
        }
    };
    let url = url.parse::<Url>()?;
    if authority.is_none() {
        let host = url.host_str().ok_or(RustamanError::RequestParsingError(
            "Host not found in HTTP Request".to_string(),
        ))?;
        let port = url
            .port_or_known_default()
            .ok_or(RustamanError::RequestParsingError(
                "Unknown http port to query".to_string(),
            ))?;
        authority = Some((host.to_string(), port));
    }
    let mut query = url.path().to_string();
    if let Some(qr) = url.query() {
        query.push_str("?");
        query.push_str(qr);
    }
    if let Some(frag) = url.fragment() {
        query.push_str("#");
        query.push_str(frag);
    }

    let scheme = Scheme::from(url.scheme());
    if let Scheme::Err(error) = scheme {
        info!("Scheme parsed from {:?}", url);
        return Err(RustamanError::RequestParsingError(error));
    }

    let mut http_frame = format!("{} {} {}\r\n", verb, query, version);
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                if unwrapped.is_empty() {
                    break;
                }
                http_frame.push_str(unwrapped);
                http_frame.push_str("\r\n");
            }
            None => break,
        }
    }
    if let Some(domain) = url.domain() {
        http_frame.push_str("Host: ");
        http_frame.push_str(domain);
        http_frame.push_str("\r\n");
    }

    let mut body = String::new();
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                body.push_str(unwrapped);
                body.push_str("\r\n");
            }
            None => break,
        }
    }
    if body.len() > 0 {
        let length = format!("{}", body.len());
        http_frame.push_str("Content-Length: ");
        http_frame.push_str(length.as_str());
        http_frame.push_str("\r\n");
    }
    if http_frame.find("\nUser-Agent:").is_none() {
        http_frame.push_str("User-Agent: Rustaman\r\n");
    }
    http_frame.push_str("Connection: close\r\n");
    http_frame.push_str("\r\n");

    if body.len() > 0 {
        http_frame.push_str(body.as_str());
    }
    let authority = authority.unwrap();
    let (host, port) = (authority.0, authority.1);
    info!("Http request built");
    Ok(HttpRequest {
        scheme,
        host,
        port,
        http_frame,
        tls_flags,
        capture,
    })
}

#[derive(Debug, Clone)]
pub struct HttpRequests {
    requests: Vec<String>,
}

fn parse_template(template: &str) -> HttpRequests {
    let requests: Vec<String> = RE_SPLIT_END_CAPTURE
        .split(template)
        .map(|request| request.to_string())
        .collect();
    debug!("{:?}", requests);
    HttpRequests { requests }
}

pub struct HttpModel {
    request: HttpRequests,
    current_request: Option<HttpRequest>,
    current_request_idx: usize,
    error: Option<RustamanError>,
    context: serde_yaml::Value,
    response: Vec<u8>,
    relm: Relm<Http>,
    stream: Option<IOStream>,
}

#[derive(Msg)]
pub enum Msg {
    StartConsuming,
    StartConsumingHttpRequest,
    Connecting(HttpRequest),
    ConnectionAquired(SocketConnection, HttpRequest),
    DisplayError(RustamanError),
    Read((Vec<u8>, usize)),
    CaptureReadDone(String),
    ReadDone(String),
    Writing(HttpRequest),
    Wrote((Vec<u8>, usize)),
}

unsafe impl Send for Msg {}

pub struct Http {
    model: HttpModel,
}

impl Update for Http {
    type Model = HttpModel;
    type ModelParam = (String, serde_yaml::Value);
    type Msg = Msg;

    fn model(relm: &Relm<Self>, params: Self::ModelParam) -> HttpModel {
        let (http_request, context) = params;
        // Fix errors
        let request = parse_template(http_request.as_str());
        let response = Vec::new();
        HttpModel {
            request,
            context,
            response,
            relm: relm.clone(),
            error: None,
            stream: None,
            current_request: None,
            current_request_idx: 0,
        }
    }

    fn subscriptions(&mut self, _relm: &Relm<Self>) {}

    fn update(&mut self, message: Msg) {
        match message {
            Msg::StartConsuming => {
                let error = mem::replace(&mut self.model.error, None);
                if let Some(err) = error {
                    self.model.relm.stream().emit(Msg::DisplayError(err));
                } else {
                    self.model
                        .relm
                        .stream()
                        .emit(Msg::StartConsumingHttpRequest);
                }
            }
            Msg::StartConsumingHttpRequest => {
                if self.model.current_request_idx < self.model.request.requests.len() {
                    info!(
                        "StartConsumingHttpRequest: {} / {}",
                        self.model.current_request_idx + 1,
                        self.model.request.requests.len()
                    );
                    let req = self
                        .model
                        .request
                        .requests
                        .get(self.model.current_request_idx)
                        .unwrap();
                    let http_request = compile_template(req.as_str(), &self.model.context)
                        .unwrap_or("".to_owned());
                    info!("{}", http_request);
                    let req = parse_request(http_request.as_str());
                    if let Err(err) = req {
                        self.model.relm.stream().emit(Msg::DisplayError(err));
                    } else {
                        // start consuming without error here
                        let req = req.unwrap();
                        self.model.current_request = Some(req.clone());
                        self.model.relm.stream().emit(Msg::Connecting(req));
                    }
                }
            }
            Msg::Connecting(req) => {
                info!("Connecting: {:?}", self.model.request.requests);
                let client = SocketClient::new();
                if req.scheme == Scheme::HTTPS {
                    client.set_tls(true);
                    client.set_tls_validation_flags(req.tls_flags);
                }
                let host = req.host.clone();
                connect_async!(
                    client,
                    connect_to_host_async(host.as_str(), req.port),
                    self.model.relm,
                    |conn| Msg::ConnectionAquired(conn, req)
                );
            }
            Msg::ConnectionAquired(connection, req) => {
                info!("Connecting to {:?}", connection);
                let stream: IOStream = connection.upcast();
                let writer = stream.get_output_stream().expect("output");
                self.model.stream = Some(stream);

                let buffer = req.http_frame.as_str().to_string().into_bytes();
                self.model.relm.stream().emit(Msg::Writing(req.clone()));
                connect_async!(
                    writer,
                    write_async(buffer, PRIORITY_DEFAULT),
                    self.model.relm,
                    |msg| Msg::Wrote(msg)
                );
            }
            // To be listened by the user.
            Msg::Read((mut buffer, size)) => {
                if size == 0 {
                    let buffer = String::from_utf8(self.model.response.clone()).unwrap();
                    self.model.current_request_idx += 1;
                    if self.model.current_request_idx == self.model.request.requests.len() {
                        self.model.relm.stream().emit(Msg::ReadDone(buffer));
                    } else {
                        self.model.relm.stream().emit(Msg::CaptureReadDone(buffer));
                    }
                } else {
                    if let Some(ref stream) = self.model.stream {
                        let reader = stream.get_input_stream().expect("input");
                        connect_async!(
                            reader,
                            read_async(vec![0; READ_SIZE], PRIORITY_DEFAULT),
                            self.model.relm,
                            Msg::Read
                        );
                    }
                }
                buffer.truncate(size);
                self.model.response.extend(&buffer);
            }
            // To be listened by the user.
            Msg::CaptureReadDone(response) | Msg::ReadDone(response) => {
                let req = mem::replace(&mut self.model.current_request, None);
                let req = req.unwrap();
                if let Some(capture) = req.capture {
                    let resp = parse_response(response.as_str());
                    if let Some(r) = resp {
                        match &mut self.model.context {
                            serde_yaml::Value::Mapping(mapping) => {
                                mapping.insert(serde_yaml::Value::String(capture.to_string()), r);
                            }
                            _ => {}
                        }
                    }
                }
                self.model.response.clear();
                self.model
                    .relm
                    .stream()
                    .emit(Msg::StartConsumingHttpRequest);
            }
            Msg::Writing(_) => (),
            Msg::Wrote(_) => {
                if let Some(ref stream) = self.model.stream {
                    let reader = stream.get_input_stream().expect("input");
                    connect_async!(
                        reader,
                        read_async(vec![0; READ_SIZE], PRIORITY_DEFAULT),
                        self.model.relm,
                        Msg::Read
                    );
                }
            }
            _ => {}
        }
    }
}

impl UpdateNew for Http {
    fn new(_relm: &Relm<Self>, model: HttpModel) -> Self {
        Http { model }
    }
}
