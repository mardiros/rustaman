// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use sourceview5::{self, prelude::*};

use crate::models::Environment;

#[derive(Debug, Clone)]
pub enum EnvironmentMsg {
    RunHttpRequest,
}

#[derive(Debug, Clone)]
pub enum EnvironmentOutput {
    RunHttpRequest,
}

pub struct EnvironmentEditor {}

impl EnvironmentEditor {}

pub struct Widgets {
    environment_id: usize,
    buffer: sourceview5::Buffer,
}

impl Widgets {
    pub fn get_environment_id(&self) -> usize {
        return self.environment_id;
    }
    pub fn get_environment(&self) -> String {
        let start_iter = self.buffer.start_iter();
        let end_iter = self.buffer.end_iter();
        let text = self.buffer.text(&start_iter, &end_iter, true);
        text.as_str().to_string()
    }
}

fn create_buffer() -> sourceview5::Buffer {
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);

    let langmngr = sourceview5::LanguageManager::default();
    let stmngr = sourceview5::StyleSchemeManager::default();

    if let Some(ref language) = langmngr.language("yaml") {
        buffer.set_language(Some(language));
    } else {
        error!("Can't find yaml.lang lang in {:?}", langmngr.search_path())
    }
    if let Some(ref scheme) = stmngr.scheme("rustaman-dark") {
        buffer.set_style_scheme(Some(scheme));
    } else {
        error!(
            "Can't find rustaman-dark.xml theme in {:?}",
            stmngr.search_path()
        )
    }
    buffer
}

impl Component for EnvironmentEditor {
    type Init = Environment;
    type Input = EnvironmentMsg;
    type Output = EnvironmentOutput;
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        environment: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = create_buffer();

        let sender = sender.output_sender().clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_evt, key, _code, mask| {
            if key == gtk::gdk::Key::Return && mask == gtk::gdk::ModifierType::CONTROL_MASK {
                error!("Emmitting {:?}", EnvironmentOutput::RunHttpRequest);
                sender.emit(EnvironmentOutput::RunHttpRequest);
                return true.into();
            }
            false.into()
        });
        let environment_source = sourceview5::View::with_buffer(&buffer);
        environment_source.set_margin_all(10);
        environment_source.add_controller(controller);
        buffer.set_text(environment.payload());

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    environment_source -> SourceView {
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                }
            }
        }

        ComponentParts {
            model: EnvironmentEditor {},
            widgets: Widgets {
                buffer,
                environment_id: environment.id(),
            },
        }
    }

    fn update(
        &mut self,
        _message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
