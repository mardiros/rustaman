// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use sourceview5::{self, prelude::*};

use crate::models::Environment;

#[derive(Debug, Clone)]
pub enum EnvironmentMsg {}

pub enum EnvironmentOutput {}

pub struct EnvironmentEditor {
    environment: Environment,
}

pub struct Widgets {}

impl Component for EnvironmentEditor {
    type Init = Environment;
    type Input = EnvironmentMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        environment: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_highlight_syntax(true);

        let langmngr = sourceview5::LanguageManager::default();
        let stmngr = sourceview5::StyleSchemeManager::default();

        let search_path = langmngr.search_path();
        debug!("{:?}", search_path);

        if let Some(ref language) = langmngr.language("yaml") {
            buffer.set_language(Some(language));
        } else {
            error!(
                "Can't find rustaman-Environment.lang lang in {:?}",
                search_path
            )
        }
        if let Some(ref scheme) = stmngr.scheme("rustaman-dark") {
            buffer.set_style_scheme(Some(scheme));
        }

        let environment_source = sourceview5::View::with_buffer(&buffer);
        environment_source.set_margin_all(10);
        buffer.set_text(environment.payload());

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                #[local_ref]
                environment_source -> SourceView {
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        }

        ComponentParts {
            model: EnvironmentEditor { environment },
            widgets: Widgets { },
        }
    }

    fn update(
        &mut self,
        _message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
    }
}
