use gdk;
use gdk::enums::key;
use gtk;
use gtk::WindowType::Toplevel;
use gtk::prelude::*;

use super::super::models::Workspace;
use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum Msg {
    Quit,
    KeyPress(gdk::EventKey),
}

pub struct Window {
    relm: Relm<Window>,
    win: gtk::Window,
    model: Workspace,
}

impl Update for Window {
    type Model = Workspace;
    type ModelParam = Workspace;
    type Msg = Msg;

    fn model(_: &Relm<Self>, workspace: Workspace) -> Workspace {
        workspace
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::KeyPress(key) => {
                let keyval = key.get_keyval();
                let keystate = key.get_state();

                if keystate.intersects(gdk::ModifierType::CONTROL_MASK) {
                    match keyval {
                        key::w => self.relm.stream().emit(Msg::Quit),
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Widget for Window {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.win.clone()
    }

    fn view(relm: &Relm<Self>, model: Workspace) -> Self {
        let window = gtk::Window::new(Toplevel);

        window.set_title(model.name());
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1280, 1024);

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        connect!(
            relm,
            window,
            connect_key_press_event(_, key),
            return (Msg::KeyPress(key.clone()), Inhibit(false))
        );

        window.show_all();
        Window {
            win: window,
            relm: relm.clone(),
            model: model,
        }
    }
}
