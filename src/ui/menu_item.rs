use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt};

use super::super::models::Request;

#[derive(Debug)]
pub enum MenuMode {
    Toggle,
    Edit,
    Renaming,
    Deleting,
}
pub struct MenuItem {
    visible: bool,
    selected: bool,
    request: Request,
    mode: MenuMode,
}

impl MenuItem {
    pub fn id(&self) -> usize {
        self.request.id()
    }
    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, value: bool) {
        self.selected = value
    }

    pub fn edit(&mut self) {
        self.mode = MenuMode::Renaming
    }

    pub fn set_name(&mut self, name: &str) {
        self.request.set_name(name);
        self.mode = MenuMode::Toggle
    }
    pub fn search(&mut self, search: &str) {
        if search.len() > 0 {
            self.visible = self
                .request
                .name()
                .to_lowercase()
                .contains(search.to_lowercase().as_str())
        } else {
            self.visible = true;
        }
        debug!(
            "Search res {} {} => {}",
            search,
            self.request.name(),
            self.visible
        )
    }
}

#[derive(Debug, Clone)]
pub enum MenuItemMsg {
    DeleteRequest,
    RenameRequest,
    CancelRenameRequest,
    ValidateRenameRequest,
}

#[derive(Debug, Clone)]
pub enum MenuItemOutput {
    DeleteRequest(usize),
    TogglingRequest(usize, bool),
    RenameRequest(usize, String),
}

pub struct MenuItemWidgets {
    root: gtk::Box,
    toggle: gtk::ToggleButton,
    edit_entry: gtk::Entry,
    toggle_container: gtk::Box,
}

impl FactoryComponent for MenuItem {
    type Init = Request;
    type Input = MenuItemMsg;
    type Output = MenuItemOutput;
    type CommandOutput = ();
    type Root = gtk::Box;
    type ParentWidget = gtk::Box;
    type Widgets = MenuItemWidgets;
    type Index = DynamicIndex;

    fn init_root(&self) -> Self::Root {
        gtk::Box::default()
    }

    fn init_model(
        request: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            request,
            selected: false,
            visible: true,
            mode: MenuMode::Toggle,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let request_id = self.request.id();

        let toggle = gtk::ToggleButton::new();
        let inner_root = gtk::Box::default();
        let toggle_container = gtk::Box::default();

        let entry_sender = sender.input_sender().clone();
        let controller = gtk::EventControllerKey::new();
        // controller.connect_im_update(
        controller.connect_key_pressed(move |_evt, key, _code, _mask| match key {
            gtk::gdk::Key::Return => {
                entry_sender.emit(MenuItemMsg::ValidateRenameRequest);
                true.into()
            }
            gtk::gdk::Key::Escape => {
                entry_sender.emit(MenuItemMsg::CancelRenameRequest);
                true.into()
            }
            _ => false.into(),
        });

        let edit_entry = gtk::Entry::new();
        edit_entry.add_controller(controller);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                #[local_ref]
                inner_root -> gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,

                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 2,
                set_hexpand: true,
                set_vexpand: false,

                #[local_ref]
                edit_entry -> gtk::Entry {
                    set_text: self.request.name(),
                    set_can_focus: true,
                    select_region: (0, self.request.name().len() as i32),
                    set_hexpand: true,
                },
                #[local_ref]
                toggle_container -> gtk::Box{
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    #[local_ref]
                    toggle -> gtk::ToggleButton {
                        set_label: self.request.name(),
                        set_hexpand: true,
                        set_focus_on_click: false,
                        connect_toggled[sender] => move |btn| {
                            sender.output(MenuItemOutput::TogglingRequest(request_id, btn.is_active())).unwrap();
                        },
                    },
                    gtk::MenuButton {
                        set_hexpand: false,

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Button {
                                    set_label: "Rename",
                                    connect_clicked => MenuItemMsg::RenameRequest,
                                },
                                gtk::Button {
                                    set_label: "Delete",
                                    connect_clicked => MenuItemMsg::DeleteRequest,
                                }
                            }
                        }
                    }
                }
            }
        }                }

        edit_entry.hide();

        MenuItemWidgets {
            root: inner_root,
            toggle,
            toggle_container,
            edit_entry,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        info!("Update {:?}", msg);
        match msg {
            MenuItemMsg::RenameRequest => self.mode = MenuMode::Edit,
            MenuItemMsg::CancelRenameRequest => self.mode = MenuMode::Toggle,
            MenuItemMsg::ValidateRenameRequest => self.mode = MenuMode::Renaming,
            MenuItemMsg::DeleteRequest => self.mode = MenuMode::Deleting,
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: FactorySender<Self>) {
        info!("Update View {:?} {}", self.mode, self.request.id());
        widgets.toggle.set_active(self.selected);
        match self.mode {
            MenuMode::Edit => {
                widgets.edit_entry.show();
                widgets.toggle_container.hide();
                widgets.edit_entry.grab_focus();
            }
            MenuMode::Toggle => {
                widgets.edit_entry.set_text(self.request.name());
                widgets.toggle.set_label(self.request.name());
                widgets.edit_entry.hide();
                widgets.toggle_container.show();
            }
            MenuMode::Renaming => {
                debug!("Renaming request");
                sender.output_sender().emit(MenuItemOutput::RenameRequest(
                    self.request.id(),
                    widgets.edit_entry.text().into(),
                ))
            }
            MenuMode::Deleting => {
                debug!("Renaming request");
                sender
                    .output_sender()
                    .emit(MenuItemOutput::DeleteRequest(self.request.id()))
            }
        }
        if self.visible {
            widgets.root.show();
        } else {
            widgets.root.hide();
        }
    }
}
