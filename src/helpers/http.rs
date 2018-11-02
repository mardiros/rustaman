use std::convert::From;


use gio::{
    IOStream, IOStreamExt, InputStreamExtManual, OutputStreamExtManual, SocketClient,
    SocketClientExt, SocketConnection,
};
use glib::source::PRIORITY_DEFAULT;
use glib::Cast;
use relm::{Relm, Update, UpdateNew};

use regex::Regex;
use url::Url;

const READ_SIZE: usize = 1024;


#[derive(Debug, PartialEq, Clone)]
pub enum Scheme {
    HTTP,
    HTTPS,
    Err(String),  // until TryFrom is stable
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
}

impl HttpRequest {
    pub fn http_frame(&self) -> &str {
        self.http_frame.as_str()
    }

    pub fn authority(&self) -> (&str, u16) {
        (self.host.as_str(), self.port)
    }

}

pub fn parse_request(request: &str) -> Result<HttpRequest, String> {
    info!("Parsing request {}", request.len());
    let mut lines = request.lines();
    let mut line = lines.next();
    loop {
        if line.is_none() {
            break;
        }
        let unwrapped = line.unwrap();
        if !unwrapped.is_empty() && !unwrapped.starts_with('#') {
            break;
        }
        debug!("Ignoring comment {}", unwrapped);
        line = lines.next();
    }
    if line.is_none() {
        return Err("! No request found".to_owned());
    }

    info!("Parsing First line {:?}", line);
    let split_verb_re: Regex = Regex::new("[ ]+").unwrap();
    let verb_url_version: Vec<&str> = split_verb_re.split(line.unwrap()).collect();
    let (verb, url, version) = match verb_url_version.len() {
        2 => (verb_url_version[0], verb_url_version[1], "HTTP/1.1"),
        3 => (
            verb_url_version[0],
            verb_url_version[1],
            verb_url_version[2],
        ),
        _ => {
            return Err(format!("! Parse error on line: {}", line.unwrap()).to_owned());
        }
    };
    let url = url.parse::<Url>().map_err(|err| format!("! {}", err))?;
    let host = url
        .host_str()
        .ok_or("! Host not found in HTTP Request".to_string())?;
    let host = host.to_string();
    let port = url
        .port_or_known_default()
        .ok_or("! Unknown http port to query".to_string())?;
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
        return Err(error)
    }

    let mut http_frame = format!("{} {} {}\r\n", verb, query, version);
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                http_frame.push_str(unwrapped);
                http_frame.push_str("\r\n");
                if unwrapped.is_empty() {
                    break;
                }
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
    http_frame.push_str("Connection: close\r\n");
    http_frame.push_str("\r\n");

    if body.len() > 0 {
        http_frame.push_str(body.as_str());
    }

    Ok(HttpRequest {
        scheme,
        host,
        port,
        http_frame,
    })
}

pub struct HttpModel {
    request: Result<HttpRequest, String>,
    response: Vec<u8>,
    relm: Relm<Http>,
    stream: Option<IOStream>,
}

#[derive(Msg)]
pub enum Msg {
    Connection(SocketConnection),
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
    type ModelParam = String;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, http_request: String) -> HttpModel {
        let request = parse_request(http_request.as_str());
        let response = Vec::new();
        HttpModel {
            request,
            response,
            relm: relm.clone(),
            stream: None,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        if let Ok(ref req) = self.model.request.as_ref() {
            let client = SocketClient::new();
            if req.scheme == Scheme::HTTPS {
                client.set_tls(true);
            }
            connect_async!(
                client,
                connect_to_host_async(req.host.as_str(), req.port),
                relm,
                Msg::Connection
            );
        }
    }

    fn update(&mut self, message: Msg) {
        match message {
            Msg::Connection(connection) => {
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
        }
    }
}

impl UpdateNew for Http {
    fn new(_relm: &Relm<Self>, model: HttpModel) -> Self {
        Http { model }
    }
}
