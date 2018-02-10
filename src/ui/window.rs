use gtk;
use gtk::WindowType::Toplevel;
use gtk::prelude::*;

use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum Msg {
    Quit,
}

pub struct Window {
    win: gtk::Window,
}

impl Update for Window {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Window {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.win.clone()
    }

    fn view(relm: &Relm<Self>, model: ()) -> Self {
        let window = gtk::Window::new(Toplevel);

        window.set_title("Rustaman");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1280, 1024);

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );
        window.show_all();
        Window { win: window }
    }
}
