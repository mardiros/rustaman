// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use sourceview5;

use crate::helpers::sourceview::create_buffer;

#[derive(Debug, Clone)]
pub enum TrafficLogMsg {
    Connecting(String),
    SendingHttpRequest(String),
    ReceivingHttpResponse(String),
}

pub struct TrafficLog {
    buffer: sourceview5::Buffer,
}

impl TrafficLog {
    fn log(&self, msg: &str) {
        let start_iter = self.buffer.start_iter();
        let end_iter = self.buffer.end_iter();
        let mut current: String = self.buffer.text(&start_iter, &end_iter, true).into();
        current.push_str(msg);
        current.push_str("\n");
        self.buffer.set_text(current.as_str());
    }
}

pub struct Widgets {}

impl Component for TrafficLog {
    type Init = ();
    type Input = TrafficLogMsg;
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
        let buffer = create_buffer("rustaman-response");
        let request_source = sourceview5::View::with_buffer(&buffer);
        request_source.set_margin_all(10);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    request_source -> SourceView {
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                }
            }
        }

        ComponentParts {
            model: TrafficLog { buffer },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            TrafficLogMsg::Connecting(host) => {
                self.log(">>> New request");
                let authority = format!("#! Authority: {}", host.as_str());
                self.log(authority.as_str());
                self.log("");
            }
            TrafficLogMsg::SendingHttpRequest(request) => {
                self.log(request.as_str());
            }
            TrafficLogMsg::ReceivingHttpResponse(response) => {
                self.log(response.as_str());
            }
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
