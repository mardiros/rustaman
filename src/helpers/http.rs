use std::convert::From;
use std::str::FromStr;

use lazy_static::lazy_static;

use gio::{
    IOStream, IOStreamExt, InputStreamExtManual, OutputStreamExtManual, SocketClient,
    SocketClientExt, SocketConnection, TlsCertificateFlags,
};
use glib::source::PRIORITY_DEFAULT;
use glib::Cast;
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
            }).collect();
        req
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

fn extract_insecure_flag(line: &str) -> bool {
    return RE_EXTRACT_INSECURE_FLAG.is_match(line);
}

pub fn parse_request(request: &str) -> RustamanResult<HttpRequest> {
    info!("Parsing request {}", request.len());

    let mut lines = request.lines();
    let mut line = lines.next();
    let mut authority: Option<(String, u16)> = None;
    let mut tls_flags = TlsCertificateFlags::all();

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
        } else if extract_insecure_flag(unwrapped) {
            tls_flags = TlsCertificateFlags::empty();
        } else {
            debug!("Ignoring comment {}", unwrapped);
        }
        line = lines.next();
    }
    if line.is_none() {
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
    Ok(HttpRequest {
        scheme,
        host,
        port,
        http_frame,
        tls_flags,
    })
}

pub struct HttpModel {
    request: RustamanResult<HttpRequest>,
    response: Vec<u8>,
    relm: Relm<Http>,
    stream: Option<IOStream>,
}

#[derive(Msg)]
pub enum Msg {
    StartHttpRequest,
    Connection(SocketConnection),
    ConnectionError(RustamanError),
    RequestParsingError(String),
    Read((Vec<u8>, usize)),
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
        let http_request =
            compile_template(http_request.as_str(), &context).unwrap_or("".to_owned());
        let request = parse_request(http_request.as_str());
        let response = Vec::new();
        HttpModel {
            request,
            response,
            relm: relm.clone(),
            stream: None,
        }
    }

    fn subscriptions(&mut self, _relm: &Relm<Self>) {}

    fn update(&mut self, message: Msg) {
        match message {
            Msg::StartHttpRequest => match self.model.request.as_ref() {
                Ok(req) => {
                    let client = SocketClient::new();
                    if req.scheme == Scheme::HTTPS {
                        client.set_tls(true);
                        client.set_tls_validation_flags(req.tls_flags);
                    }
                    connect_async!(
                        client,
                        connect_to_host_async(req.host.as_str(), req.port),
                        self.model.relm,
                        Msg::Connection,
                        |err: gdk::Error| Msg::ConnectionError(RustamanError::from(err.clone()))
                    );
                }
                Err(err) => {
                    error!("Request is in error: {:?}", self.model.request);
                    let err = format!("{}", err);
                    self.model
                        .relm
                        .stream()
                        .emit(Msg::RequestParsingError(err.to_owned()));
                }
            },
            Msg::Connection(connection) => {
                info!("Connecting to {:?}", connection);
                let stream: IOStream = connection.upcast();
                let writer = stream.get_output_stream().expect("output");
                self.model.stream = Some(stream);
                let _ = self.model.request.as_ref().map(|req| {
                    let http_frame = req.http_frame.clone();
                    let buffer = http_frame.into_bytes();
                    self.model.relm.stream().emit(Msg::Writing(req.clone()));
                    connect_async!(
                        writer,
                        write_async(buffer, PRIORITY_DEFAULT),
                        self.model.relm,
                        |msg| Msg::Wrote(msg)
                    );
                });
            }
            // To be listened by the user.
            Msg::Read((mut buffer, size)) => {
                if size == 0 {
                    let buffer = String::from_utf8(self.model.response.clone()).unwrap();
                    self.model.relm.stream().emit(Msg::ReadDone(buffer));
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
            Msg::ReadDone(_) => (),
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
            Msg::RequestParsingError(err) => {
                error!("{:?}", err);
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
