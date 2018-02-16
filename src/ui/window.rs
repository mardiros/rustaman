use gdk;
use gdk::enums::key;
use gtk::{self, Orientation, WindowPosition, WindowType};
use gtk::prelude::*;
use glib::translate::ToGlib;
use relm::{Component, ContainerWidget, Relm, Update, Widget};

use super::super::models::{Query, Workspace};
use super::menu::{Menu, Msg as MenuMsg};

#[derive(Msg)]
pub enum Msg {
    CreateRequest,
    ToggleRequest(usize, bool),
    RequestNameChanged(usize, String),
    Quit,
    KeyPress(gdk::EventKey),
}

pub struct Model {
    workspace: Workspace,
    id_generator: usize,
    id: usize,
}

impl Model {
    pub fn name(&self) -> &str {
        self.workspace.name()
    }
    pub fn queries(&self) -> &[Query] {
        self.workspace.queries()
    }
    pub fn next_id(&mut self) -> usize {
        self.id_generator += 1;
        return self.id_generator;
    }
}

pub struct Window {
    model: Model,
    menu: Component<Menu>,
    window: gtk::Window,
    hbox: gtk::Box,
    relm: Relm<Window>,
}

impl Update for Window {
    type Model = Model;
    type ModelParam = Workspace;
    type Msg = Msg;

    fn model(_: &Relm<Self>, workspace: Workspace) -> Model {
        let id_generator = workspace.queries().len();
        Model {
            workspace: workspace,
            id_generator: id_generator,
            id: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreateRequest => {
                let id = self.model.next_id();
                self.menu.stream().emit(MenuMsg::CreateRequest(id))
            }
            Msg::ToggleRequest(id, active) => {
                if active {
                    self.model.id = id;
                } else if self.model.id == id {
                    self.model.id = 0;
                }
            }
            Msg::RequestNameChanged(id, name) => {}
            Msg::Quit => gtk::main_quit(),
            Msg::KeyPress(key) => {
                let keyval = key.get_keyval();
                let keystate = key.get_state();

                if keystate.intersects(gdk::ModifierType::CONTROL_MASK) {
                    match keyval {
                        key::w => self.relm.stream().emit(Msg::Quit),
                        key::n => self.relm.stream().emit(Msg::CreateRequest),
                        _ => {}
                    }
                } else {
                    match keyval {
                        key::F2 => {
                            if self.model.id > 0 {
                                self.menu
                                    .stream()
                                    .emit(MenuMsg::RenameRequest(self.model.id))
                            }
                        }
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
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        let window = gtk::Window::new(WindowType::Toplevel);

        window.set_title(model.name());
        window.set_border_width(10);
        window.set_position(WindowPosition::Center);
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

        let settings = gtk::Settings::get_default().unwrap();
        let use_dark = true;
        settings.set_long_property(
            "gtk-application-prefer-dark-theme",
            use_dark.to_glib() as _,
            "",
        );

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);
        let queries = model.queries().to_vec();
        let menu = hbox.add_widget::<Menu, _>(relm, queries);
        window.set_hexpand(true);
        window.set_vexpand(true);

        connect!(
            menu@MenuMsg::NewRequest,
            relm,
            Msg::CreateRequest
        );

        connect!(
            menu@MenuMsg::ToggleRequest(idx, active),
            relm,
            Msg::ToggleRequest(idx, active)
        );

        connect!(
            menu@MenuMsg::RequestNameChanged(idx, ref name),
            relm,
            Msg::RequestNameChanged(idx, name.to_owned())
        );

        window.add(&hbox);
        window.show_all();
        Window {
            model: model,
            menu: menu,
            window: window,
            hbox: hbox,
            relm: relm.clone(),
        }
    }
}
