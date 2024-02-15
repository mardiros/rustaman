// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use serde_json;
use sourceview5::{self, prelude::*};

use crate::helpers::sourceview::create_buffer;

fn prettify_js(payload: &str) -> Result<String, serde_json::Error> {
    let obj: serde_json::Value = serde_json::from_str(payload)?;
    Ok(serde_json::to_string_pretty(&obj).unwrap())
}

#[derive(Debug, Clone)]
pub enum ResponseBodyMsg {
    ReceivingHttpResponse(String),
    ReceivingError(String),
}

pub struct ResponseBody {
    buffer: sourceview5::Buffer,
}

impl ResponseBody {
    fn log_error(&self, error: &str) {
        self.buffer.set_text(error);
    }
    fn log_response(&self, response: &str) {
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
        }
        let response = if is_json && has_content {
            match prettify_js(text.as_str()) {
                Ok(pretty) => pretty,
                Err(_) => text,
            }
        } else {
            text
        };

        self.buffer.set_text(response.as_str());
    }
}

pub struct Widgets {}

impl Component for ResponseBody {
    type Init = ();
    type Input = ResponseBodyMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        _request: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = create_buffer("json");
        let response_view = sourceview5::View::with_buffer(&buffer);
        response_view.set_show_line_numbers(true);
        response_view.set_margin_all(10);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    response_view -> SourceView {
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                }
            }
        }

        ComponentParts {
            model: ResponseBody { buffer },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        debug!("Updating response body");
        match message {
            ResponseBodyMsg::ReceivingHttpResponse(response) => {
                self.log_response(response.as_str())
            }
            ResponseBodyMsg::ReceivingError(error) => self.log_error(error.as_str()),
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
