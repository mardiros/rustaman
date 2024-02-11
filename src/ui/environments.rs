// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};

use crate::models::{Environment, Environments};
use crate::ui::environ_editor::EnvironmentEditor;

#[derive(Debug, Clone)]
pub enum EnvironmentsMsg {}

pub enum EnvironmentsOutput {}

pub struct EnvironmentsTabs {
    environments: Environments,
}

pub struct Widgets {}

impl Component for EnvironmentsTabs {
    type Init = Environments;
    type Input = EnvironmentsMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        environments: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let notebook = gtk::Notebook::new();

        for environment in environments.iter() {
            let editor = EnvironmentEditor::builder().launch(environment.clone());
            notebook.append_page(
                editor.widget(),
                Some(&gtk::Button::with_label(environment.name())),
            );
        }
        notebook.append_page(
            &gtk::Box::new(gtk::Orientation::Horizontal, 5),
            Some(&gtk::Button::with_label("+")),
        );

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                set_hexpand: true,
                set_vexpand: true,
                #[local_ref]
                notebook -> gtk::Notebook {
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        }

        ComponentParts {
            model: EnvironmentsTabs { environments },
            widgets: Widgets {},
        }
    }

    fn update(
        &mut self,
        _message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
