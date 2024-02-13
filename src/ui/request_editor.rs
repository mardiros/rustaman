// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use sourceview5::{self, prelude::*};

use crate::helpers::sourceview::create_buffer;
use crate::models::Request;

#[derive(Debug, Clone)]
pub enum RequestMsg {
    RequestChanged(Request),
}

#[derive(Debug, Clone)]
pub enum RequestOutput {
    RunHttpRequest,
}

pub struct RequestEditor {
    request: Option<Request>,
}

impl RequestEditor {
    pub fn request_id(&self) -> Option<usize> {
        return self.request.as_ref().map(|r| r.id());
    }
}

pub struct Widgets {
    buffer: sourceview5::Buffer,
}

impl Widgets {
    pub fn get_template(&self) -> String {
        let start_iter = self.buffer.start_iter();
        let end_iter = self.buffer.end_iter();
        let text = self.buffer.text(&start_iter, &end_iter, true);
        text.as_str().to_string()
    }
}

impl Component for RequestEditor {
    type Init = Option<Request>;
    type Input = RequestMsg;
    type Output = RequestOutput;
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        request: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = create_buffer("rustaman-request");
        let request_source = sourceview5::View::with_buffer(&buffer);

        let sender = sender.output_sender().clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_evt, key, _code, mask| {
            if key == gtk::gdk::Key::Return && mask == gtk::gdk::ModifierType::CONTROL_MASK {
                error!("Emmitting {:?}", RequestOutput::RunHttpRequest);
                sender.emit(RequestOutput::RunHttpRequest);
                return true.into();
            }
            false.into()
        });

        // request_source.set_margin_all(10);
        request_source.set_show_line_numbers(true);
        request_source.add_controller(controller);

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
            model: RequestEditor { request },
            widgets: Widgets { buffer },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            RequestMsg::RequestChanged(request) => {
                self.request = Some(request);
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        if let Some(request) = self.request.as_ref() {
            widgets.buffer.set_text(request.template());
        }
    }
}
