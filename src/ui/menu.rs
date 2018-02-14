use std::collections::HashMap;

use gtk::{self, IconSize, Orientation};
use gtk::prelude::*;

use relm::{Component, ContainerWidget, Relm, Update, Widget};

use super::menu_item::{MenuItem, Msg as MenuItemMsg};
use super::super::models::Queries;

pub struct Model {
    queries: Queries,
    current: usize,
}

#[derive(Msg)]
pub enum Msg {
    NewRequest,
    CreateRequest(usize),
    ToggleRequest(usize, bool),
}

pub struct Menu {
    relm: Relm<Menu>,
    vbox: gtk::Box,
    items: HashMap<usize, Component<MenuItem>>,
    model: Model,
}

impl Update for Menu {
    type Model = Model;
    type ModelParam = Queries;
    type Msg = Msg;

    fn model(_: &Relm<Self>, queries: Queries) -> Model {
        Model {
            queries: queries,
            current: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreateRequest(idx) => {
                let item = self.vbox.add_widget::<MenuItem, _>(&self.relm, idx);

                connect!(
                    item@MenuItemMsg::ToggleRequest(id, active),
                    self.relm,
                    Msg::ToggleRequest(id, active)
                );

                item.stream().emit(MenuItemMsg::SetActive(true));
                self.items.insert(idx, item);
            }
            Msg::ToggleRequest(idx, active) => {
                if active {
                    let current = self.model.current;
                    if current > 0 && current != idx {
                        let item = self.items.get_mut(&self.model.current).unwrap();
                        item.stream().emit(MenuItemMsg::SetActive(false));
                    }
                    self.model.current = idx;
                }
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

        let items = HashMap::new();
        // TODO: Fill items with model.queries here

        vbox.add(&hbox);
        vbox.show_all();
        Menu {
            vbox: vbox,
            relm: relm.clone(),
            items: items,
            model: model,
        }
    }
}
