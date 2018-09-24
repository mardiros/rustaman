use gtk::prelude::*;
use gtk::{self, Align, Justification, Orientation};
use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum Msg {
    Showing,
    Hiding,
}

pub struct HelpBox {
    vbox: gtk::Box,
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
                self.vbox.show();
            }
            Msg::Hiding => {
                self.vbox.hide();
            }
        }
    }
}

impl Widget for HelpBox {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        info!("Creating Help Box widget");
        let vbox = gtk::Box::new(Orientation::Vertical, 10);
        vbox.set_hexpand(true);
        vbox.set_vexpand(true);
        vbox.set_valign(Align::Center);

        fn create_label(vbox: &gtk::Box, shortcut: &str, title: &str) {
            let hbox = gtk::Box::new(Orientation::Horizontal, 0);
            let label = gtk::Label::new(shortcut);
            label.set_justify(Justification::Right);
            label.set_hexpand(true);
            hbox.add(&label);
            let label = gtk::Label::new(title);
            label.set_justify(Justification::Left);
            label.set_hexpand(true);
            hbox.add(&label);
            hbox.set_hexpand(true);
            vbox.add(&hbox);
        }

        create_label(&vbox, "CTRL+N", "new request");
        create_label(&vbox, "CTRL+P", "search request");
        create_label(&vbox, "CTRL+Enter", "Execute request");

        vbox.show_all();
        HelpBox { vbox }
    }
}
