use gdk;
use gdk::enums::key;
use gtk::{self, Orientation, WindowPosition, WindowType};
use gtk::prelude::*;
use glib::translate::ToGlib;
use relm::{Component, ContainerWidget, Relm, Update, Widget};

use super::super::models::{Request, Template, Workspace};
use super::menu::{Menu, Msg as MenuMsg};
use super::request_editor::{Msg as EditorMsg, RequestEditor};

#[derive(Msg)]
pub enum Msg {
    CreateRequest,
    ToggleRequest(usize, bool),
    RequestNameChanged(usize, String),
    RequestTemplateChanged(usize, Template),
    Quit,
    KeyPress(gdk::EventKey),
}

pub struct Model {
    workspace: Workspace,
    current: usize,
}

impl Model {
    pub fn name(&self) -> &str {
        self.workspace.name()
    }
    pub fn requests(&self) -> &[Request] {
        self.workspace.requests()
    }
    pub fn current_request(&self) -> Option<&Request> {
        self.workspace.request(self.current)
    }
    pub fn create_request(&mut self) -> &Request {
        self.workspace.create_request()
    }
}

pub struct Window {
    model: Model,
    menu: Component<Menu>,
    window: gtk::Window,
    editor_box: gtk::Box,
    relm: Relm<Window>,
    request_editor: Component<RequestEditor>,
}

impl Update for Window {
    type Model = Model;
    type ModelParam = Workspace;
    type Msg = Msg;

    fn model(_: &Relm<Self>, workspace: Workspace) -> Model {
        Model {
            workspace: workspace,
            current: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreateRequest => {
                let request = self.model.create_request();
                self.menu
                    .stream()
                    .emit(MenuMsg::CreateRequest(request.clone()))
            }
            Msg::ToggleRequest(id, active) => {
                if self.model.current > 0 {
                    self.request_editor
                        .stream()
                        .emit(EditorMsg::SaveRequest(self.model.current));
                }
                if active {
                    self.editor_box.show();
                    self.model.current = id;
                    let req = self.model.current_request().unwrap(); // XXX
                    self.editor_box.show();
                    self.request_editor
                        .stream()
                        .emit(EditorMsg::TemplateChanged(req.template().to_owned()));
                } else if self.model.current == id {
                    self.editor_box.hide();
                    self.model.current = 0;
                }
            }
            Msg::RequestNameChanged(id, name) => {
                self.model.workspace.set_request_name(id, name.as_str());
            }
            Msg::RequestTemplateChanged(id, template) => {
                info!("Save Template Changes {} {}", id, template);
                self.model
                    .workspace
                    .set_request_template(id, template.as_str());
            }
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
                            if self.model.current > 0 {
                                self.menu
                                    .stream()
                                    .emit(MenuMsg::RenameRequest(self.model.current))
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
        let requests = model.requests().to_vec();
        let menu = hbox.add_widget::<Menu, _>(relm, requests);
        window.set_hexpand(true);
        window.set_vexpand(true);

        connect!(
            menu@MenuMsg::NewRequest,
            relm,
            Msg::CreateRequest
        );

        connect!(
            menu@MenuMsg::ToggleRequest(id, active),
            relm,
            Msg::ToggleRequest(id, active)
        );

        connect!(
            menu@MenuMsg::RequestNameChanged(id, ref name),
            relm,
            Msg::RequestNameChanged(id, name.to_owned())
        );
        hbox.show_all();

        let editor_box = gtk::Box::new(Orientation::Horizontal, 0);
        let editor = editor_box.add_widget::<RequestEditor, _>(relm, ());
        connect!(
            editor@EditorMsg::Save(id, ref template),
            relm,
            Msg::RequestTemplateChanged(id, template.to_owned())
        );
        hbox.add(&editor_box);

        window.add(&hbox);
        window.show();
        Window {
            model: model,
            menu: menu,
            window: window,
            editor_box: editor_box,
            request_editor: editor,
            relm: relm.clone(),
        }
    }
}
