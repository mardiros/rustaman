use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use super::super::models::Request;

pub struct MenuItem {
    selected: bool,
    request: Request,
}

impl MenuItem {
    pub fn id(&self) -> usize {
        self.request.id()
    }

    pub fn name(&self) -> &str {
        self.request.name()
    }

    pub fn active(&self) -> bool {
        self.request.active()
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, value: bool) {
        self.selected = value
    }

}

#[derive(Debug, Clone)]
pub enum MenuItemMsg {
    // SetActive(bool),
    // EntryKeyPress(gdk::EventKey),
    // RenamingRequest,
    // Renaming(usize, String),
    // Deleting(usize),
    // FilteringName(String),
}

#[derive(Debug, Clone)]
pub enum MenuItemOutput {
    DeleteRequest(usize),
    TogglingRequest(usize, bool),
}

pub struct MenuItemWidgets {
    toggle: gtk::ToggleButton,
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
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
            }
        }
        root
    }

    fn init_model(
        request: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { request, selected: false }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let request_id = self.request.id();
        let entry = gtk::Entry::new();
        let toggle = gtk::ToggleButton::new();
        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: false,

                #[local_ref]
                entry -> gtk::Entry {
                    set_text: self.request.name(),
                    set_can_focus: true,
                    select_region: (0, self.request.name().len() as i32),
                    set_hexpand: true,
                },

                gtk::Box{
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
                                },
                                gtk::Button {
                                    set_label: "Delete",
                                }
                            }
                        }
                    }
                }
            }
        }
        entry.hide();

        MenuItemWidgets {toggle}
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        info!("Update {:?}", msg);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: FactorySender<Self>) {
        widgets.toggle.set_active(self.selected)
    }
}
