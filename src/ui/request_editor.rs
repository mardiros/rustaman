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
    ToggleOff,
}

#[derive(Debug, Clone)]
pub enum RequestOutput {
    RunHttpRequest,
    SaveHttpRequest(usize, String),
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
    request_id: usize,
    buffer: sourceview5::Buffer,
    request_source_container: gtk::ScrolledWindow,
    help_container: gtk::Box,
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
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = create_buffer("rustaman-request");
        let request_source = sourceview5::View::with_buffer(&buffer);

        let sender = sender.output_sender().clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_evt, key, _code, mask| {
            if key == gtk::gdk::Key::Return && mask == gtk::gdk::ModifierType::CONTROL_MASK {
                sender.emit(RequestOutput::RunHttpRequest);
                return true.into();
            }
            false.into()
        });

        let help_container = gtk::Box::default();
        let request_source_container = gtk::ScrolledWindow::default();
        // request_source.set_margin_all(10);
        request_source.set_show_line_numbers(true);
        request_source.add_controller(controller);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                // inline_css: "background-color: #444",

                #[local_ref]
                help_container -> gtk::Box {
                    set_spacing: 15,
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                    set_hexpand_set: false,
                    set_margin_start: 100,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_markup: "<big>ctrl+n: new request</big>"
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_markup: "<big>ctrl+p: search request</big>"
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_markup: "<big>ctrl+p: ctrl+Enter</big>"
                    },
                },
                #[local_ref]
                request_source_container -> gtk::ScrolledWindow {
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
        request_source_container.hide();

        ComponentParts {
            model: RequestEditor { request },
            widgets: Widgets {
                buffer,
                request_source_container,
                help_container,
                request_id: 0,
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            RequestMsg::RequestChanged(request) => {
                self.request = Some(request);
            }
            RequestMsg::ToggleOff => {
                self.request = None;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        if let Some(request) = self.request.as_ref() {
            if widgets.request_id > 0 {
                sender.output_sender().emit(RequestOutput::SaveHttpRequest(
                    widgets.request_id,
                    widgets.get_template(),
                ));
            }

            widgets.help_container.hide();
            widgets.request_source_container.show();
            widgets.buffer.set_text(request.template());
            widgets.request_id = request.id()
        } else {
            widgets.request_source_container.hide();
            widgets.help_container.show();
        }
    }
}
