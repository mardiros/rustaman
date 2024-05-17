use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;

use lazy_static::lazy_static;

use super::super::errors::{RustamanError, RustamanResult};
use super::super::models::Environment;
use super::handlebars;
use regex::Regex;
use reqwest::Method;

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

fn extract_authority_from_directive(line: &str) -> Option<(String, u16)> {
    let resp = RE_EXTRACT_AUTHORITY_FROM_DIRECTIVE
        .captures(line)
        .and_then(|cap| {
            let host = cap
                .name("host")
                .map(|host| host.as_str().trim_start_matches('[').trim_end_matches(']'));
            let port = cap
                .name("port")
                .map(|port| FromStr::from_str(port.as_str()).unwrap());
            Some((host?.to_string(), port?))
        });
    resp
}

fn extract_capture_name(line: &str) -> Option<String> {
    let resp = RE_EXTRACT_CAPTURE.captures(line).and_then(|cap| {
        let cap = cap.name("capture");
        cap.map(|capture| capture.as_str().to_string())
    });
    resp
}

fn extract_insecure_flag(line: &str) -> bool {
    RE_EXTRACT_INSECURE_FLAG.is_match(line)
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
    // pub scheme: Scheme,
    pub method: Method,
    pub url: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
    // host: String,
    // port: u16,
    pub http_frame: String,
    pub verify_cert: bool,
    pub capture: Option<String>,
}

impl HttpRequest {
    pub fn verify_cert(&self) -> bool {
        self.verify_cert
    }
    pub fn method(&self) -> Method {
        self.method.clone()
    }
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    pub fn body(&self) -> Option<String> {
        match &self.body {
            Some(b) => Some(b.to_string()),
            None => None,
        }
    }

    pub fn http_frame(&self) -> &str {
        self.http_frame.as_str()
    }
    /// Obfusface the http_frame
    pub fn obfuscate(&self, env: &Environment) -> HttpRequest {
        let mut req = self.clone();
        let s = env.obfuscated_string();
        let _: Vec<_> = s
            .iter()
            .map(|x| {
                let obf = format!("{}...", &x[0..3]);
                req.http_frame = req.http_frame.replace(x.as_str(), obf.as_str());
            })
            .collect();
        req
    }
}

fn parse_request(request: &str) -> RustamanResult<HttpRequest> {
    info!("Parsing request {}", request.len());

    let mut lines = request.lines();
    let mut line = lines.next();
    // let mut authority: Option<(String, u16)> = None;
    let mut verify_cert = true;
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
            // authority = Some(auth);
        } else if let Some(cap) = extract_capture_name(unwrapped) {
            capture = Some(cap);
        } else if extract_insecure_flag(unwrapped) {
            verify_cert = false;
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
    let (verb, url, _version) = match verb_url_version.len() {
        2 => (verb_url_version[0], verb_url_version[1], "HTTP/1.1"),
        3 => (
            verb_url_version[0],
            verb_url_version[1],
            verb_url_version[2],
        ),
        _ => {
            error!("Parse error on line: {}", line.unwrap());
            return Err(RustamanError::RequestParsingError(format!(
                "Parse error on line: {}",
                line.unwrap()
            )));
        }
    };
    let method = Method::from_str(verb).unwrap();

    let mut http_frame = line.unwrap().to_string();
    http_frame.push_str("\r\n");
    let mut headers = HashMap::new();
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                if unwrapped.is_empty() {
                    break;
                }
                let header = unwrapped.split_once(':');
                if let Some((key, val)) = header {
                    headers.insert(key.to_string(), val.to_string());
                    http_frame.push_str(unwrapped);
                    http_frame.push_str("\r\n");
                } else {
                    // FIXME: multiline header
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
    http_frame.push_str("\r\n");

    let mut body = String::new();
    loop {
        let line = lines.next();
        match line {
            Some(unwrapped) => {
                body.push_str(unwrapped);
                body.push_str("\r\n");

                http_frame.push_str(unwrapped);
                http_frame.push_str("\r\n");
            }
            None => break,
        }
    }

    info!("Http request built");
    Ok(HttpRequest {
        // scheme,
        method,
        url: url.to_string(),
        headers,
        body: if body.is_empty() { None } else { Some(body) },
        // host,
        // port,
        http_frame,
        verify_cert,
        capture,
    })
}

pub fn split_template(template: &str) -> Vec<String> {
    let requests: Vec<String> = RE_SPLIT_END_CAPTURE
        .split(template)
        .map(|request| request.to_string())
        .collect();
    debug!("{:?}", requests);
    requests
}

pub fn load_template(template: &str, environ: &Environment) -> RustamanResult<HttpRequest> {
    let context = environ.parsed_payload()?;
    let template_rendered = handlebars::render_template(template, &context)?;
    parse_request(template_rendered.as_str())
}
