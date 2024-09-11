// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use relm4_icons::icon_names;

use crate::models::{Environment, Environments};
use crate::ui::environ_editor::{EnvironmentEditor, EnvironmentOutput};

#[derive(Debug, Clone)]
pub enum EnvironmentsMsg {
    NewEnvironment,
    CancelCreate,
    CreateEnvironment(String),
    EnvironmentCreated(Environment),
    RenamingEnvironment(usize),
    RenameEnvironment(usize, String),
    CancelRename(usize),
    EnvironmentRenamed(usize, String),
    DeleteEnvironment(usize),
    EnvironmentDeleted(usize),
    Initialized,
}

#[derive(Debug, Clone)]
pub enum EnvironmentsOutput {
    RunHttpRequest,
    CreateEnvironment(String),
    RenameEnvironment(usize, String),
    DeleteEnvironment(usize),
}
pub enum NewEnvironmentMode {
    Append,
    Creating,
}

pub struct EnvironmentsTabs {
    mode: NewEnvironmentMode,
    notebook: gtk::Notebook,
    tab_labels: Vec<(gtk::Label, gtk::Entry)>,
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
    type Output = EnvironmentsOutput;
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
        let tab_labels = Vec::new();

        relm4::view! {
            #[local_ref]
            tab_label -> gtk::Box {
                #[local_ref]
                new_tab_btn -> gtk::Button {
                    set_icon_name: icon_names::TAB_NEW,
                    connect_clicked => EnvironmentsMsg::NewEnvironment,
                },
                #[local_ref]
                new_tab_entry -> gtk::Entry {
                    set_width_chars: 12,
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
                tab_labels,
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
            EnvironmentsMsg::Initialized => {
                self.notebook.set_current_page(Some(0));
            }
            EnvironmentsMsg::NewEnvironment => self.mode = NewEnvironmentMode::Creating,
            EnvironmentsMsg::CancelCreate => self.mode = NewEnvironmentMode::Append,
            EnvironmentsMsg::CreateEnvironment(environment) => sender
                .output_sender()
                .emit(EnvironmentsOutput::CreateEnvironment(environment)),
            EnvironmentsMsg::EnvironmentCreated(environment) => {
                let editor = EnvironmentEditor::builder()
                    .launch(environment.clone())
                    .forward(sender.output_sender(), |msg| match msg {
                        EnvironmentOutput::RunHttpRequest => EnvironmentsOutput::RunHttpRequest,
                    });

                let tab_label = gtk::Box::default();
                let env_id = environment.id();
                let tab_label_labl = gtk::Label::default();
                let tab_label_entry = gtk::Entry::new();

                let entry_sender = sender.input_sender().clone();
                let controller = gtk::EventControllerKey::new();
                controller.connect_key_pressed(move |_evt, key, _code, _mask| match key {
                    gtk::gdk::Key::Escape => {
                        entry_sender.emit(EnvironmentsMsg::CancelRename(env_id));
                        true.into()
                    }
                    _ => false.into(),
                });
                tab_label_entry.add_controller(controller);

                tab_label_entry.hide();
                relm4::view! {
                    #[local_ref]
                    tab_label -> gtk:: Box {
                        #[local_ref]
                        tab_label_labl -> gtk::Label {
                            set_margin_end: 10,
                            set_label: environment.name(),
                        },
                        #[local_ref]
                        tab_label_entry -> gtk::Entry {
                            set_text: environment.name(),
                            set_width_chars: 12,
                            connect_activate[sender] => move |entry| {
                                let buffer = entry.buffer();
                                sender.input(EnvironmentsMsg::RenameEnvironment(env_id, buffer.text().into()));
                                buffer.delete_text(0, None);
                            }
                        },
                        gtk::MenuButton {
                            set_hexpand: false,

                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::Button {
                                        set_label: "Rename",
                                        connect_clicked => EnvironmentsMsg::RenamingEnvironment(env_id)
                                    },
                                    gtk::Button {
                                        set_label: "Delete",
                                        connect_clicked => EnvironmentsMsg::DeleteEnvironment(env_id)
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
                self.notebook.emit_change_current_page(page_num as i32);
                self.tab_labels.push((tab_label_labl, tab_label_entry));

                self.mode = NewEnvironmentMode::Append;
            }
            EnvironmentsMsg::RenamingEnvironment(env_id) => {
                let index = self
                    .editors
                    .iter()
                    .position(|ed| ed.widgets().get_environment_id() == env_id);
                if let Some(page_num) = index {
                    let (lbl, entry) = self.tab_labels.get_mut(page_num).unwrap();
                    lbl.hide();
                    entry.show();
                    entry.grab_focus();
                }
            }
            EnvironmentsMsg::CancelRename(env_id) => {
                let index = self
                    .editors
                    .iter()
                    .position(|ed| ed.widgets().get_environment_id() == env_id);
                if let Some(page_num) = index {
                    let (lbl, entry) = self.tab_labels.get_mut(page_num).unwrap();
                    lbl.show();
                    entry.hide();
                }
            }
            EnvironmentsMsg::RenameEnvironment(env_id, name) => sender
                .output_sender()
                .emit(EnvironmentsOutput::RenameEnvironment(env_id, name)),
            EnvironmentsMsg::EnvironmentRenamed(env_id, name) => {
                let index = self
                    .editors
                    .iter()
                    .position(|ed| ed.widgets().get_environment_id() == env_id);
                if let Some(page_num) = index {
                    let (lbl, entry) = self.tab_labels.get_mut(page_num).unwrap();
                    lbl.set_label(name.as_str());
                    lbl.show();
                    entry.hide();
                }
            }
            EnvironmentsMsg::DeleteEnvironment(env_id) => sender
                .output_sender()
                .emit(EnvironmentsOutput::DeleteEnvironment(env_id)),
            EnvironmentsMsg::EnvironmentDeleted(env_id) => {
                let index = self
                    .editors
                    .iter()
                    .position(|ed| ed.widgets().get_environment_id() == env_id);
                if let Some(page_num) = index {
                    self.notebook.remove_page(Some(page_num as u32));
                }
                self.notebook.emit_change_current_page(0);
            }
        }
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
