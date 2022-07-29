use gdk;
use gdk::keys::constants;
use gtk::prelude::*;
use gtk::{self, Orientation};
use relm::{connect, Relm, Update, Widget};

use super::super::models::Request;

pub struct Model {
    request: Request,
}

impl Model {
    pub fn id(&self) -> usize {
        self.request.id()
    }

    pub fn name(&self) -> &str {
        self.request.name()
    }

    pub fn active(&self) -> bool {
        self.request.active()
    }
}

#[derive(Msg)]
pub enum Msg {
    TogglingRequest(usize, bool),
    SetActive(bool),
    EntryKeyPress(gdk::EventKey),
    RenamingRequest,
    Renaming(usize, String),
    Deleting(usize),
    FilteringName(String),
}

pub struct MenuItem {
    hbox: gtk::Box,
    displaybox: gtk::Box,
    toggle_btn: gtk::ToggleButton,
    entry: gtk::Entry,
    relm: Relm<MenuItem>,
    model: Model,
}

impl Update for MenuItem {
    type Model = Model;
    type ModelParam = Request;
    type Msg = Msg;

    fn model(_: &Relm<Self>, request: Request) -> Model {
        Model {
            request,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::RenamingRequest => {
                self.displaybox.hide();
                self.entry.show();
                self.entry.grab_focus();
            }

            Msg::SetActive(active) => {
                if self.entry.is_visible() {
                    self.entry.grab_focus();
                } else {
                    self.toggle_btn.set_active(active);
                }
            }

            Msg::EntryKeyPress(key) => {
                let keyval = key.get_keyval();
                match keyval {
                    constants::Return => {
                        let name = self.entry.get_text();
                        self.toggle_btn.set_label(name.as_str());
                        self.entry.hide();
                        self.displaybox.show();
                        self.model.request.set_name(name.as_str());
                        self.relm
                            .stream()
                            .emit(Msg::Renaming(self.model.id(), name.to_owned()))
                    }
                    constants::Escape => {
                        let name = self.model.name();
                        self.entry.set_text(name);
                        self.entry.hide();
                        self.displaybox.show();
                        self.relm
                            .stream()
                            .emit(Msg::Renaming(self.model.id(), name.to_owned()))
                    }
                    _ => {}
                }
            }

            Msg::FilteringName(filter) => {
                // don't filter new entry
                if self.entry.is_visible() {
                    return;
                }
                if self.model.name().to_lowercase().contains(filter.as_str()) {
                    self.displaybox.show()
                } else {
                    self.displaybox.hide()
                }
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
        info!("Creating menu item widget {:?}", model.request);
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);

        let entry = gtk::Entry::new();
        entry.set_text(model.name());
        entry.set_can_focus(true);
        entry.select_region(0, model.name().len() as i32);
        connect!(
            relm,
            entry,
            connect_key_press_event(_, key),
            return (Msg::EntryKeyPress(key.clone()), Inhibit(false))
        );
        entry.set_hexpand(true);
        hbox.add(&entry);

        let displaybox = gtk::Box::new(Orientation::Horizontal, 0);
        displaybox.set_hexpand(true);

        let toggle_btn = gtk::ToggleButton::with_label(model.name());
        toggle_btn.set_hexpand(true);
        toggle_btn.set_focus_on_click(false);
        toggle_btn.set_relief(gtk::ReliefStyle::Half);
        toggle_btn.show();
        displaybox.add(&toggle_btn);

        let menu = gtk::Menu::new();
        let rename = gtk::MenuItem::with_label("Rename");
        menu.append(&rename);
        let delete = gtk::MenuItem::with_label("Delete");
        menu.append(&delete);
        rename.show();
        delete.show();
        let combo_btn = gtk::MenuButton::new();
        combo_btn.set_popup(Some(&menu));
        combo_btn.set_use_popover(true);
        combo_btn.show();
        displaybox.add(&combo_btn);
        hbox.add(&displaybox);

        let model_id = model.id();
        connect!(relm, rename, connect_activate(_), Msg::RenamingRequest);
        connect!(relm, delete, connect_activate(_), Msg::Deleting(model_id));

        let model_id = model.id();
        connect!(
            relm,
            toggle_btn,
            connect_clicked(btn),
            Msg::TogglingRequest(model_id, btn.get_active())
        );
        if model.active() {
            displaybox.show();
        } else {
            entry.show();
        }
        hbox.show();
        MenuItem {
            hbox,
            displaybox,
            toggle_btn,
            entry,
            model,
            relm: relm.clone(),
        }
    }
}
