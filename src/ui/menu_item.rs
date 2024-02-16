use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt};

use super::super::models::Request;

pub struct MenuItem {
    visible: bool,
    selected: bool,
    request: Request,
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
pub enum MenuItemMsg {}

#[derive(Debug, Clone)]
pub enum MenuItemOutput {
    DeleteRequest(usize),
    TogglingRequest(usize, bool),
}

pub struct MenuItemWidgets {
    root: gtk::Box,
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
        let entry = gtk::Entry::new();
        let toggle = gtk::ToggleButton::new();
        let inner_root = gtk::Box::default();
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
        }                }

        entry.hide();

        MenuItemWidgets {
            root: inner_root,
            toggle,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        info!("Update {:?}", msg);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: FactorySender<Self>) {
        if self.visible {
            widgets.root.show();
        } else {
            widgets.root.hide();
        }
        widgets.toggle.set_active(self.selected)
    }
}
