use gtk::{self, Align, Justification, Orientation};
use gtk::prelude::*;
use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum Msg {
    Showing,
    Hiding,
}

pub struct HelpBox {
    hbox: gtk::Box,
}

impl Update for HelpBox {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Showing => {
                self.hbox.show();
            }
            Msg::Hiding => {
                self.hbox.hide();
            }
        }
    }
}

impl Widget for HelpBox {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        info!("Creating Help Box widget");
        let hbox = gtk::Box::new(Orientation::Vertical, 0);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);
        hbox.set_valign(Align::Center);

        let label = gtk::Label::new("CTRL+n new request");
        label.set_justify(Justification::Center);
        hbox.add(&label);
        hbox.show_all();
        HelpBox { hbox: hbox }
    }
}
