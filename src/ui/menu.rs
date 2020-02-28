use std::collections::HashMap;
use std::slice::Iter;

use gdk;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{self, IconSize, Orientation};
use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};

use super::super::models::{Request, Requests};
use super::menu_item::{MenuItem, Msg as MenuItemMsg};

pub struct Model {
    requests: Requests,
    current: usize,
}

impl Model {
    pub fn requests_iter(&self) -> Iter<Request> {
        self.requests.iter()
    }
}

#[derive(Msg)]
pub enum Msg {
    NewRequest,
    CreatingRequest(Request),
    RenamingRequest(usize),
    TogglingRequest(usize, bool),
    Renaming(usize, String),
    Deleting(usize),
    Deleted(usize),
    RequestingFilteringMenu,
    SearchEntryPressingKey(gdk::EventKey),
}

pub struct Menu {
    relm: Relm<Menu>,
    main_box: gtk::Box,
    vbox: gtk::Box,
    search: gtk::SearchEntry,
    items: HashMap<usize, Component<MenuItem>>,
    model: Model,
}

impl Update for Menu {
    type Model = Model;
    type ModelParam = Requests;
    type Msg = Msg;

    fn model(_: &Relm<Self>, requests: Requests) -> Model {
        Model {
            requests,
            current: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreatingRequest(request) => {
                info!("Create request in menu {:?}", request);
                let id = request.id();
                let req_active = request.active();
                let item = self.vbox.add_widget::<MenuItem>(request);

                connect!(
                    item@MenuItemMsg::TogglingRequest(id, active),
                    self.relm,
                    Msg::TogglingRequest(id, active)
                );
                connect!(
                    item@MenuItemMsg::Renaming(id, ref name),
                    self.relm,
                    Msg::Renaming(id, name.to_owned())
                );
                connect!(
                    item@MenuItemMsg::Deleting(id),
                    self.relm,
                    Msg::Deleting(id)
                );

                if !req_active {
                    item.stream().emit(MenuItemMsg::SetActive(true));
                }
                self.items.insert(id, item);
            }

            Msg::Deleted(id) => {
                self.model.current = 0;
                let item = self.items.remove(&id);
                if item.is_some() {
                    self.vbox.remove_widget::<MenuItem>(item.unwrap());
                }
            }
            Msg::TogglingRequest(id, active) => {
                info!("Toggle request in menu {} {}", id, active);
                if active {
                    let current = self.model.current;
                    if current > 0 && current != id {
                        let item = self.items.get_mut(&self.model.current).unwrap();
                        item.stream().emit(MenuItemMsg::SetActive(false));
                    }
                    self.model.current = id;
                } else if self.model.current == id {
                    self.model.current = 0;
                }
            }
            Msg::RenamingRequest(id) => {
                info!("Rename request in menu {}", id);
                let item = self.items.get_mut(&self.model.current);
                if item.is_some() {
                    item.unwrap().stream().emit(MenuItemMsg::RenamingRequest);
                } else {
                    error!("Cannot rename unexisting request #{}", id);
                }
            }
            Msg::RequestingFilteringMenu => {
                info!("Requesting filter in menu");
                self.search.grab_focus();
            }
            Msg::SearchEntryPressingKey(key) => {
                let keyval = key.get_keyval();
                let name = match keyval {
                    key::Escape => {
                        self.search.set_text("");
                        "".to_owned()
                    }
                    _ => self.search.get_text().unwrap().to_owned(),
                };

                for menuitem in self.items.values_mut() {
                    menuitem
                        .stream()
                        .emit(MenuItemMsg::FilteringName(name.to_lowercase()))
                }
            }
            _ => {}
        }
    }
}

impl Widget for Menu {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.main_box.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating menu widget");
        let main_box = gtk::Box::new(Orientation::Vertical, 0);
        let vbox = gtk::Box::new(Orientation::Vertical, 0);
        vbox.set_hexpand(true);

        let searchbox = gtk::Box::new(Orientation::Horizontal, 0);
        searchbox.set_hexpand(true);
        searchbox.set_margin_top(5);
        searchbox.set_margin_bottom(10);

        //let add_request = gtk::Button::new_with_label("+");
        let add_request = gtk::Button::new();
        let add_image = gtk::Image::new_from_icon_name(Some("document-new"), IconSize::Button.into());
        add_request.set_relief(gtk::ReliefStyle::Half);
        add_request.set_focus_on_click(false);
        add_request.add(&add_image);
        searchbox.add(&add_request);

        connect!(relm, add_request, connect_clicked(_), Msg::NewRequest);

        let search = gtk::SearchEntry::new();
        searchbox.pack_start(&search, true, true, 0);

        connect!(
            relm,
            search,
            connect_key_press_event(_, key),
            return (Msg::SearchEntryPressingKey(key.clone()), Inhibit(false))
        );

        main_box.add(&searchbox);

        let items = HashMap::new();
        for request in model.requests_iter() {
            if request.active() {
                relm.stream().emit(Msg::CreatingRequest(request.clone()));
            }
        }

        let scrollbox = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        scrollbox.add(&vbox);
        scrollbox.show();

        main_box.pack_start(&scrollbox, true, true, 0);
        main_box.show_all();

        Menu {
            main_box,
            vbox,
            search,
            items,
            model,
            relm: relm.clone(),
        }
    }
}
