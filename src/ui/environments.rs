// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};

use crate::models::Environments;
use crate::ui::environ_editor::{EnvironmentEditor, EnvironmentOutput};

#[derive(Debug, Clone)]
pub enum EnvironmentsMsg {
    RunHttpRequest,
}

pub struct EnvironmentsTabs {}

impl EnvironmentsTabs {}

pub struct Widgets {
    notebook: gtk::Notebook,
    editors: Vec<Controller<EnvironmentEditor>>,
}

impl Widgets {
    pub fn environment_id(&self) -> Option<usize> {
        if let Some(idx) = self.notebook.current_page() {
            return Some(self.editors[idx as usize].widgets().get_environment_id());
        }
        None
    }
    pub fn get_environment(&self) -> String {
        if let Some(idx) = self.notebook.current_page() {
            return self.editors[idx as usize].widgets().get_environment();
        }
        return "".to_string();
    }
}

impl Component for EnvironmentsTabs {
    type Init = Environments;
    type Input = EnvironmentsMsg;
    type Output = EnvironmentsMsg;
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        environments: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let notebook = gtk::Notebook::new();

        let mut editors = Vec::new();
        for environment in environments.iter() {
            let editor = EnvironmentEditor::builder()
                .launch(environment.clone())
                .forward(sender.output_sender(), |msg| match msg {
                    EnvironmentOutput::RunHttpRequest => EnvironmentsMsg::RunHttpRequest,
                });
            notebook.append_page(
                editor.widget(),
                Some(&gtk::Button::with_label(environment.name())),
            );
            editors.push(editor);
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
            model: EnvironmentsTabs {},
            widgets: Widgets { notebook, editors },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        // we forward all the message to the window
        sender.input_sender().emit(message)
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
