use gtk::{self, Orientation, IconSize};
use gtk::prelude::*;

use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum Msg {
}

pub struct Menu {
    //relm: Relm<Menu>,
    vbox: gtk::Box,
}

impl Update for Menu {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {}
    }
}

impl Widget for Menu {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        let vbox = gtk::Box::new(Orientation::Vertical, 0);

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);

        //let add_request = gtk::Button::new_with_label("+");
        let add_request = gtk::Button::new();
        let add_image = gtk::Image::new_from_icon_name(
            "document-new", IconSize::Button.into());
        add_request.set_relief(gtk::ReliefStyle::Half);
        add_request.set_focus_on_click(false);
        add_request.add(&add_image);
        hbox.add(&add_request);

        let search = gtk::SearchEntry::new();
        hbox.add(&search);

        vbox.add(&hbox);
        vbox.show_all();
        Menu {
            vbox: vbox,
            //relm: relm.clone(),
        }
    }
}
