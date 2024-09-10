// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use relm4_icons::icon_names;

use crate::models::Request;
use crate::ui::menu_item::MenuItemOutput;

use super::menu_item::{MenuItem, MenuMode};

#[derive(Debug, Clone)]
pub enum SideBarMsg {
    NewRequest,
    CreateRequest(Request),
    RegisterRequest(Request),
    TogglingRequest(usize, bool),
    DeleteRequest(usize),
    SearchingRequest,
    SearchRequest(String),
    RenameRequest(usize, String),
    RequestRenamed(usize, String),
    RequestDeleted(usize),
}

#[derive(Debug, Clone)]
pub enum SideBarOutput {
    NewRequest,
    TogglingRequest(usize),
    DeleteRequest(usize),
    RenameRequest(usize, String),
    ToggleOff,
}

pub struct SideBar {
    menu_items: FactoryVecDeque<MenuItem>,
    search_entry: gtk::SearchEntry,
}

impl SideBar {}

pub struct Widgets {}

impl Widgets {}

impl Component for SideBar {
    type Init = Vec<Request>;
    type Input = SideBarMsg;
    type Output = SideBarOutput;
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        requests: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let menu_items =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output| match output {
                    MenuItemOutput::DeleteRequest(request_id) => {
                        SideBarMsg::DeleteRequest(request_id)
                    }
                    MenuItemOutput::TogglingRequest(request_id, active) => {
                        SideBarMsg::TogglingRequest(request_id, active)
                    }
                    MenuItemOutput::RenameRequest(request_id, name) => {
                        SideBarMsg::RenameRequest(request_id, name)
                    }
                });

        for request in requests {
            if request.active() {
                sender
                    .input_sender()
                    .send(SideBarMsg::RegisterRequest(request.clone()))
                    .unwrap();
            }
        }

        let menu_items_container: &gtk::Box = menu_items.widget();
        let search_entry = gtk::SearchEntry::new();
        let search_sender = sender.input_sender().clone();
        search_entry.connect_search_changed(move |entry| {
            search_sender.emit(SideBarMsg::SearchRequest(entry.text().into()));
        });

        let new_request_btn = gtk::Button::new();

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                set_orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    set_margin_all: 2,
                    set_orientation: gtk::Orientation::Horizontal,

                    #[local_ref]
                    new_request_btn -> gtk::Button {
                        set_icon_name: icon_names::DOCUMENT_ADD_REGULAR,
                        connect_clicked => SideBarMsg::NewRequest
                    },
                    #[local_ref]
                    search_entry -> gtk::SearchEntry {
                        set_vexpand: false,
                        set_hexpand: true,
                        set_valign: gtk::Align::Fill,
                        // inline_css: "border: 2px solid blue",
                    },
                },

                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    menu_items_container -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                    }
                }
            }
        }
        search_entry.show();
        ComponentParts {
            model: SideBar {
                menu_items,
                search_entry,
            },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        let mut menu_items_guard = self.menu_items.guard();
        match &message {
            SideBarMsg::NewRequest => {
                sender.output_sender().emit(SideBarOutput::NewRequest);
            }
            SideBarMsg::CreateRequest(request) => {
                debug!("Create Request");
                menu_items_guard.push_back((request.clone(), MenuMode::Edit));
            }
            SideBarMsg::RegisterRequest(request) => {
                menu_items_guard.push_back((request.clone(), MenuMode::Toggle));
            }
            SideBarMsg::RenameRequest(request_id, name) => {
                sender
                    .output_sender()
                    .emit(SideBarOutput::RenameRequest(*request_id, name.clone()));
            }
            SideBarMsg::TogglingRequest(request_id, active) => {
                if *active {
                    info!("Activating request {}", request_id);
                    for item in menu_items_guard.iter_mut() {
                        if item.id() == *request_id {
                            item.set_selected(true);
                        }
                        if item.selected() && item.id() != *request_id {
                            item.set_selected(false);
                        }
                    }
                    sender
                        .output_sender()
                        .emit(SideBarOutput::TogglingRequest(*request_id))
                } else {
                    for item in menu_items_guard.iter_mut() {
                        if item.id() == *request_id {
                            debug!("Delelecting the current request");
                            item.set_selected(false);
                            sender.output_sender().emit(SideBarOutput::ToggleOff)
                        }
                    }
                }
            }
            SideBarMsg::SearchingRequest => {
                self.search_entry.grab_focus();
            }
            SideBarMsg::SearchRequest(search) => {
                for item in menu_items_guard.iter_mut() {
                    item.search(search.as_str());
                }
            }
            SideBarMsg::RequestRenamed(request_id, name) => {
                for item in menu_items_guard.iter_mut() {
                    if item.id() == *request_id {
                        item.set_name(name);
                    }
                }
            }
            SideBarMsg::DeleteRequest(request_id) => {
                let index = menu_items_guard
                    .iter()
                    .position(|req| req.id() == *request_id);
                if let Some(idx) = index {
                    menu_items_guard.remove(idx);
                }
                sender
                    .output_sender()
                    .emit(SideBarOutput::DeleteRequest(*request_id))
            }
            _ => (),
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        info!("update_view")
    }
}
