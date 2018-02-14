use gtk::{self, Orientation};
use gtk::prelude::*;

use relm::{Relm, Update, Widget};

pub struct Model {
    id: usize,
}

#[derive(Msg)]
pub enum Msg {
    ToggleRequest(usize, bool),
    SetActive(bool),
}

pub struct MenuItem {
    hbox: gtk::Box,
    toggle_btn: gtk::ToggleButton,
}

impl Update for MenuItem {
    type Model = Model;
    type ModelParam = usize;
    type Msg = Msg;

    fn model(_: &Relm<Self>, request_id: usize) -> Model {
        Model { id: request_id }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::SetActive(active) => {
                self.toggle_btn.set_active(active);
            }
            _ => {}
        }
    }
}

impl Widget for MenuItem {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);

        let toggle_btn = gtk::ToggleButton::new_with_label(format!("Req #{}", &model.id).as_str());
        toggle_btn.set_hexpand(true);
        toggle_btn.set_focus_on_click(false);
        toggle_btn.set_relief(gtk::ReliefStyle::Half);
        hbox.add(&toggle_btn);

        connect!(
            relm,
            toggle_btn,
            connect_clicked(btn),
            Msg::ToggleRequest(model.id, btn.get_active())
        );

        hbox.show_all();
        MenuItem {
            hbox: hbox,
            toggle_btn: toggle_btn,
            //relm: relm.clone(),
        }
    }
}
