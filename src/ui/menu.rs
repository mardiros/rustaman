use std::collections::HashMap;
use std::slice::Iter;

use gtk::{self, IconSize, Orientation};
use gtk::prelude::*;
use relm::{Component, ContainerWidget, Relm, Update, Widget};

use super::menu_item::{MenuItem, Msg as MenuItemMsg};
use super::super::models::{Request, Requests};

pub struct Model {
    requests: Requests,
    current: usize,
}

impl Model {
    pub fn requests_iter(&self) -> Iter<Request> {
        return self.requests.iter();
    }
}

#[derive(Msg)]
pub enum Msg {
    NewRequest,
    CreatingRequest(Request),
    RenamingRequest(usize),
    TogglingRequest(usize, bool),
    RequestNameChanged(usize, String),
    RequestingFilteringMenu,
}

pub struct Menu {
    relm: Relm<Menu>,
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
            requests: requests,
            current: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreatingRequest(request) => {
                info!("Create request in menu {:?}", request);
                let id = request.id();
                let req_active = request.active();
                let item = self.vbox.add_widget::<MenuItem, _>(&self.relm, request);

                connect!(
                    item@MenuItemMsg::TogglingRequest(id, active),
                    self.relm,
                    Msg::TogglingRequest(id, active)
                );
                connect!(
                    item@MenuItemMsg::RequestNameChanged(id, ref name),
                    self.relm,
                    Msg::RequestNameChanged(id, name.to_owned())
                );
                if !req_active {
                    item.stream().emit(MenuItemMsg::SetActive(true));
                }
                self.items.insert(id, item);
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
            _ => {}
        }
    }
}

impl Widget for Menu {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating menu widget");
        let vbox = gtk::Box::new(Orientation::Vertical, 0);
        vbox.set_hexpand(false);

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);

        //let add_request = gtk::Button::new_with_label("+");
        let add_request = gtk::Button::new();
        let add_image = gtk::Image::new_from_icon_name("document-new", IconSize::Button.into());
        add_request.set_relief(gtk::ReliefStyle::Half);
        add_request.set_focus_on_click(false);
        add_request.add(&add_image);
        hbox.add(&add_request);

        connect!(relm, add_request, connect_clicked(_), Msg::NewRequest);

        let search = gtk::SearchEntry::new();
        hbox.add(&search);
        vbox.add(&hbox);

        let items = HashMap::new();
        for request in model.requests_iter() {
            relm.stream().emit(Msg::CreatingRequest(request.clone()));
        }
        vbox.show_all();
        Menu {
            vbox: vbox,
            relm: relm.clone(),
            search: search,
            items: items,
            model: model,
        }
    }
}
