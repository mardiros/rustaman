// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};

use crate::models::Request;
use crate::ui::menu_item::MenuItemOutput;

use super::menu_item::MenuItem;

#[derive(Debug, Clone)]
pub enum SideBarMsg {
    CreateRequest(Request),
    TogglingRequest(usize, bool),
    DeleteRequest(usize),
    SearchRequest(String),
}

pub struct SideBar {
    menu_items: FactoryVecDeque<MenuItem>,
}

impl SideBar {}

pub struct Widgets {}

impl Widgets {}

impl Component for SideBar {
    type Init = Vec<Request>;
    type Input = SideBarMsg;
    type Output = SideBarMsg;
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        requests: Self::Init,
        root: &Self::Root,
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
                });

        for request in requests {
            if request.active() {
                sender
                    .input_sender()
                    .send(SideBarMsg::CreateRequest(request.clone()))
                    .unwrap();
            }
        }

        let menu_items_container: &gtk::Box = menu_items.widget();
        let search_entry = gtk::SearchEntry::new();
        let sender = sender.input_sender().clone();
        search_entry.connect_search_changed(move |entry| {
            sender.emit(SideBarMsg::SearchRequest(entry.text().into()));
        });

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                set_orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    set_margin_all: 2,
                    set_orientation: gtk::Orientation::Horizontal,

                    #[local_ref]
                    search_entry -> gtk::SearchEntry {
                        set_vexpand: false,
                        set_hexpand: true,
                        set_valign: gtk::Align::Fill,
                        // inline_css: "border: 2px solid blue",
                    }
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
            model: SideBar { menu_items },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        let mut menu_items_guard = self.menu_items.guard();
        match &message {
            SideBarMsg::CreateRequest(request) => {
                menu_items_guard.push_back(request.clone());
            }
            SideBarMsg::TogglingRequest(request_id, active) => {
                info!("toggling request {:?}. active {}", request_id, active);
                if *active {
                    for item in menu_items_guard.iter_mut() {
                        if item.id() == *request_id {
                            item.set_selected(true);
                        }
                        if item.selected() && item.id() != *request_id {
                            item.set_selected(false);
                        }
                    }
                }
            }
            SideBarMsg::SearchRequest(search) => {
                error!("~~ {}", search);
                for item in menu_items_guard.iter_mut() {
                    item.search(search.as_str());
                }
            }
            _ => (),
        }
        sender.output_sender().emit(message)
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        info!("update_view")
    }
}
