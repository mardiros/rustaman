use std::vec::Vec;

use regex::Regex;

use cabot::request::Request;
use cabot::{Client, RequestBuilder};

pub struct RequestRunner {
    client: Client,
}

impl RequestRunner {
    pub fn new() -> Self {
        RequestRunner {
            client: Client::new(),
        }
    }

    pub fn parse_request(&self, request: &str) -> Result<Request, String> {
        info!("Parsing request {}", request.len());
        let mut lines = request.lines();
        let mut line = lines.next();
        loop {
            if line.is_none() {
                break;
            }
            let unwrapped = line.unwrap();
            if unwrapped.len() > 0 && !unwrapped.starts_with("#") {
                break;
            }
            debug!("Ignoring comment {}", unwrapped);
            line = lines.next();
        }
        if line.is_none() {
            return Err("No request found".to_owned());
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
                return Err(format!("Parse error on line: {}", line.unwrap()).to_owned());
            }
        };
        let mut rb = RequestBuilder::new(url)
            .set_http_method(verb)
            .set_http_version(version);

        loop {
            let line = lines.next();
            match line {
                Some(unwrapped) => {
                    if unwrapped.len() == 0 {
                        break;
                    }
                    rb = rb.add_header(unwrapped);
                }
                None => match rb.build() {
                    Ok(res) => return Ok(res),
                    Err(err) => return Err(format!("{:?}", err)),
                },
            }
        }

        let mut body = String::new();
        loop {
            let line = lines.next();
            match line {
                Some(unwrapped) => {
                    if unwrapped.len() == 0 {
                        break;
                    }
                    body.push_str(unwrapped);
                }
                None => match rb.build() {
                    Ok(res) => return Ok(res),
                    Err(err) => return Err(format!("{:?}", err)),
                },
            }
        }

        info!("Pushing Request body");
        rb = rb.set_body_as_str(body.as_str());
        match rb.build() {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn run_request(&self, request: &str) -> String {
        let mut result = String::new();
        for line in request.lines() {
            result.push_str("> ");
            result.push_str(line);
            result.push('\n');
        }
        result.push('\n');
        result.push('\n');
        let request = self.parse_request(request);
        match request {
            Ok(ref req) => {
                info!("Running the request");
                let resp = self.client.execute(req);
                match resp {
                    Err(err) => {
                        result.push_str(format!("ERROR: {:?}", err).as_str());
                    }
                    Ok(resp) => {
                        result.push_str("< ");
                        result.push_str(resp.status_line());
                        result.push('\n');

                        for line in resp.headers().iter() {
                            result.push_str("< ");
                            result.push_str(line);
                            result.push('\n');
                        }
                        let body = resp.body_as_string()
                            .unwrap_or_else(|err| format!("{:?}", err));
                        if body.len() > 0 {
                            result.push_str("<\n");
                            result.push_str(body.as_str());
                        }
                    }
                }
                result
            }
            Err(err) => err,
        }
    }
}
