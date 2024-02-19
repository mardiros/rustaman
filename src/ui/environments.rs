// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use relm4_icons::icon_name;

use crate::models::{Environment, Environments};
use crate::ui::environ_editor::{EnvironmentEditor, EnvironmentOutput};

#[derive(Debug, Clone)]
pub enum EnvironmentsMsg {
    RunHttpRequest,
    NewEnvironment,
    CancelCreate,
    CreateEnvironment(String),
    EnvironmentCreated(Environment),
    DeletingEnvironment(usize),
    EnvironmentDeleted(usize),
    Initialized,
}

pub enum NewEnvironmentMode {
    Append,
    Creating,
}

pub struct EnvironmentsTabs {
    mode: NewEnvironmentMode,
    notebook: gtk::Notebook,
    editors: Vec<Controller<EnvironmentEditor>>,
}

impl EnvironmentsTabs {
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

pub struct Widgets {
    new_tab_btn: gtk::Button,
    new_tab_entry: gtk::Entry,
}

impl Widgets {}

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
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let notebook = gtk::Notebook::new();

        let editors = Vec::new();
        for environment in environments.iter() {
            if environment.active() {
                sender
                    .input_sender()
                    .emit(EnvironmentsMsg::EnvironmentCreated(environment.clone()));
            }
        }

        let new_tab_entry = gtk::Entry::new();
        let entry_sender = sender.input_sender().clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_evt, key, _code, _mask| match key {
            gtk::gdk::Key::Escape => {
                entry_sender.emit(EnvironmentsMsg::CancelCreate);
                true.into()
            }
            _ => false.into(),
        });
        new_tab_entry.add_controller(controller);

        let new_tab_btn = gtk::Button::new();
        let tab_label = gtk::Box::default();

        relm4::view! {
            #[local_ref]
            tab_label -> gtk::Box {
                #[local_ref]
                new_tab_btn -> gtk::Button {
                    set_icon_name: icon_name::TAB_NEW,
                    connect_clicked => EnvironmentsMsg::NewEnvironment,
                },
                #[local_ref]
                new_tab_entry -> gtk::Entry {
                    set_hexpand: true,
                    connect_activate[sender] => move |entry| {
                        let buffer = entry.buffer();
                        sender.input(EnvironmentsMsg::CreateEnvironment(buffer.text().into()));
                        buffer.delete_text(0, None);
                    }
                }
            }
        }
        new_tab_entry.hide();

        notebook.append_page(
            &gtk::Box::new(gtk::Orientation::Horizontal, 5),
            Some(&tab_label),
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
        sender.input_sender().emit(EnvironmentsMsg::Initialized);

        ComponentParts {
            model: EnvironmentsTabs {
                mode: NewEnvironmentMode::Append,
                notebook,
                editors,
            },
            widgets: Widgets {
                new_tab_btn,
                new_tab_entry,
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        // we forward all the message to the window
        match message.clone() {
            EnvironmentsMsg::Initialized => self.notebook.set_page(0),
            EnvironmentsMsg::NewEnvironment => self.mode = NewEnvironmentMode::Creating,
            EnvironmentsMsg::CancelCreate => self.mode = NewEnvironmentMode::Append,
            EnvironmentsMsg::EnvironmentCreated(environment) => {
                let editor = EnvironmentEditor::builder()
                    .launch(environment.clone())
                    .forward(sender.output_sender(), |msg| match msg {
                        EnvironmentOutput::RunHttpRequest => EnvironmentsMsg::RunHttpRequest,
                    });

                let tab_label = gtk::Box::default();
                let env_id = environment.id();
                relm4::view! {
                    #[local_ref]
                    tab_label -> gtk:: Box {
                        gtk::Button {
                            set_label: environment.name(),
                        },
                        gtk::MenuButton {
                            set_hexpand: false,

                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    // gtk::Button {
                                    //     set_label: "Rename",
                                    // },
                                    gtk::Button {
                                        set_label: "Delete",
                                        connect_clicked => EnvironmentsMsg::DeletingEnvironment(env_id)
                                    }
                                }
                            }
                        }
                    }
                }
                let page_num = self.notebook.insert_page(
                    editor.widget(),
                    Some(&tab_label),
                    Some(self.notebook.n_pages() - 1),
                );
                self.editors.push(editor);
                self.notebook.set_page(page_num as i32);

                self.mode = NewEnvironmentMode::Append;
            }
            EnvironmentsMsg::EnvironmentDeleted(env_id) => {
                let index = self
                    .editors
                    .iter()
                    .position(|ed| ed.widgets().get_environment_id() == env_id);
                if let Some(page_num) = index {
                    self.notebook.remove_page(Some(page_num as u32));
                }
                self.notebook.set_page(0);
            }
            _ => {}
        }
        sender.output_sender().emit(message)
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        match self.mode {
            NewEnvironmentMode::Append => {
                widgets.new_tab_btn.show();
                widgets.new_tab_entry.hide();
            }
            NewEnvironmentMode::Creating => {
                widgets.new_tab_btn.hide();
                widgets.new_tab_entry.show();
                widgets.new_tab_entry.grab_focus();
            }
        }
    }
}
