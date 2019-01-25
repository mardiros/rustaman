use std::collections::HashMap;
use std::slice::Iter;
use std::vec::Vec;

use gdk;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{self, IconSize, Orientation, ReliefStyle, ScrolledWindow};
use relm::{connect, connect_stream, Relm, Update, Widget};
use serde_yaml;
use sourceview::prelude::*;
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView};

use super::super::errors::RustamanError;
use super::super::models::{Environment, Environments};

pub struct Model {
    current: u32,
    environments: Environments,
}

impl Model {
    pub fn environments_iter(&self) -> Iter<Environment> {
        self.environments.iter()
    }
}

#[derive(Msg)]
pub enum Msg {
    FetchingEnvironment,
    FetchedEnvironment(serde_yaml::Value),
    FetchedEnvironmentFailed(RustamanError),
    SavingEnvironment(usize, String),
    NewEntryPressingKey(gdk::EventKey),
    RequestingNewEnvironment,
    CreatingNewTabPageButton,
    CreatingEnvironment(String),
    AppendingEnvironment(Environment),
    EnvironmentCreated(Environment),
    TogglingEnvironmentIndex(u32),
    TogglingEnvironment(usize),
    DeletingEnvironment(usize),
    EnvironmentDeleted(usize),
}

pub struct EnvironEditor {
    main_box: gtk::Box,
    notebook: gtk::Notebook,
    environ_sources: HashMap<u32, (usize, String, ScrolledWindow, SourceView)>,
    relm: Relm<EnvironEditor>,
    plus_tab: (gtk::Box, gtk::Box),
    entry_tab: (gtk::Box, gtk::Box),
    entry: gtk::Entry,
    model: Model,
}

impl EnvironEditor {
    fn get_text(&self, index: u32) -> Option<String> {
        info!("{:?}", self.environ_sources);
        let &(_, _, _, ref environ_source) = self
            .environ_sources
            .get(&index)
            .expect("Should be a valid tab page index");

        let buffer = environ_source.get_buffer().unwrap();
        let start_iter = buffer.get_start_iter();
        let end_iter = buffer.get_end_iter();
        buffer.get_text(&start_iter, &end_iter, true)
    }

    fn get_current_text(&self) -> Option<String> {
        let current = self.model.current;
        self.get_text(current)
    }

    fn get_current_id(&self) -> usize {
        let current = self.model.current;
        let &(id, _, _, _) = self
            .environ_sources
            .get(&current)
            .expect("Should be a valid tab page index");
        id
    }
}

impl Update for EnvironEditor {
    type Model = Model;
    type ModelParam = Vec<Environment>;
    type Msg = Msg;

    fn model(_: &Relm<Self>, environments: Vec<Environment>) -> Model {
        Model {
            current: 0u32,
            environments,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::FetchingEnvironment => {
                let payload = match self.get_current_text() {
                    Some(data) => data,
                    None => "".to_owned(),
                };
                let params: serde_yaml::Result<serde_yaml::Value> = serde_yaml::from_str(&payload);
                match params {
                    Ok(params) => {
                        self.relm.stream().emit(Msg::FetchedEnvironment(params));
                    }
                    Err(err) => {
                        info!("Yaml Error {:?}", err);
                        self.relm
                            .stream()
                            .emit(Msg::FetchedEnvironmentFailed(RustamanError::from(err)));
                    }
                }

                self.relm.stream().emit(Msg::SavingEnvironment(
                    self.get_current_id(),
                    payload.to_owned(),
                ));
            }
            Msg::RequestingNewEnvironment => {
                info!("Detach Plus");
                self.notebook.detach_tab(&self.plus_tab.1);
                info!("Attach Entry");
                let _index = self
                    .notebook
                    .append_page(&self.entry_tab.1, Some(&self.entry_tab.0));
                self.entry.grab_focus();
            }
            Msg::NewEntryPressingKey(key) => {
                let keyval = key.get_keyval();
                match keyval {
                    key::Return => {
                        let name = self.entry.get_text().unwrap().to_owned();
                        self.entry.set_text("");
                        self.relm.stream().emit(Msg::CreatingEnvironment(name));
                    }
                    key::Escape => {
                        info!("Detach Entry");
                        self.notebook.detach_tab(&self.entry_tab.1);
                        info!("Attach Plus");
                        let _index = self
                            .notebook
                            .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
                    }
                    _ => {}
                }
            }
            Msg::AppendingEnvironment(env) => {
                let env_id = env.id();
                let name = env.name();
                let payload = env.payload();
                let tab = {
                    let close_image =
                        gtk::Image::new_from_icon_name("window-close", IconSize::Button.into());
                    let button = gtk::Button::new();
                    let label = gtk::Label::new(name);
                    let tab = gtk::Box::new(Orientation::Horizontal, 0);

                    button.set_relief(ReliefStyle::None);
                    button.set_focus_on_click(false);
                    button.add(&close_image);
                    connect!(
                        self.relm,
                        button,
                        connect_clicked(_),
                        Msg::DeletingEnvironment(env_id)
                    );

                    tab.pack_start(&label, false, false, 0);
                    tab.pack_start(&button, false, false, 0);
                    tab.show_all();
                    tab
                };

                let (tab_page, environ_source) = {
                    let langmngr = LanguageManager::get_default().unwrap();
                    let lang = langmngr.get_language("yaml").unwrap();

                    let stylemngr = StyleSchemeManager::get_default().unwrap();
                    let style = stylemngr.get_scheme("solarized-dark").unwrap();

                    let buffer = sourceview::Buffer::new_with_language(&lang);
                    buffer.set_style_scheme(&style);
                    buffer.set_text(payload);

                    let environ_source = SourceView::new_with_buffer(&buffer);
                    environ_source.set_show_line_numbers(true);
                    environ_source.set_hexpand(true);
                    environ_source.set_vexpand(true);
                    environ_source.show();

                    let tab_page = ScrolledWindow::new(None, None);
                    tab_page.set_hexpand(true);
                    tab_page.set_vexpand(true);
                    tab_page.add(&environ_source);
                    tab_page.show();
                    (tab_page, environ_source)
                };

                connect!(
                    self.relm,
                    environ_source,
                    connect_key_press_event(_, key),
                    return Inhibit(
                        key.get_state().intersects(gdk::ModifierType::CONTROL_MASK)
                            && match key.get_keyval() {
                                key::Return => true,
                                _ => false,
                            }
                    )
                );

                let index = self.notebook.append_page(&tab_page, Some(&tab));
                info!("Insert Environ id {} for {}", index, name);
                self.environ_sources
                    .insert(index, (env.id(), name.to_owned(), tab_page, environ_source));
            }

            Msg::EnvironmentCreated(env) => {
                info!("Detach Add new tab");
                self.notebook.detach_tab(&self.entry_tab.1);
                info!("Append env");
                self.update(Msg::AppendingEnvironment(env));
                info!("Attach Add new tab");
                let index = self
                    .notebook
                    .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
                info!("new tab index: {}", index);
            }

            Msg::TogglingEnvironmentIndex(idx) => {
                info!("Switch to page {}", idx);
                self.model.current = idx;
                let tab_data = self
                    .environ_sources
                    .get(&idx);
                if let Some(&(ref id, _, _, _)) = tab_data {
                    self.relm.stream().emit(Msg::TogglingEnvironment(*id))
                }
                else {
                    info!("Current env has been deleted, swith to the first valid");
                    self.notebook.set_current_page(Some(0));
                }
            }
            Msg::CreatingNewTabPageButton => {
                let _index = self
                    .notebook
                    .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
            }
            Msg::EnvironmentDeleted(id) => {
                fn get_index(
                    id: usize,
                    envs: &HashMap<u32, (usize, String, ScrolledWindow, SourceView)>,
                ) -> Option<u32> {
                    for (index, &(env_id, _, _, _)) in envs.iter() {
                        if id == env_id {
                            return Some(*index);
                        }
                    }
                    None
                }
                let index = get_index(id, &self.environ_sources)
                    .expect("Invalid index while deleting environment");
                let (_, _, tab, _) = self
                    .environ_sources
                    .remove(&index)
                    .expect("Invalid index while deleting environment");
                self.notebook.detach_tab(&tab);
            }
            _ => {}
        }
    }
}

impl Widget for EnvironEditor {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.main_box.clone()
    }

    fn init_view(&mut self) {}

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating Environ widget");

        let main_box = gtk::Box::new(Orientation::Horizontal, 0);
        let notebook = gtk::Notebook::new();

        for env in model.environments_iter() {
            if env.active() {
                relm.stream().emit(Msg::AppendingEnvironment(env.clone()));
            }
        }
        let environ_sources: HashMap<u32, (usize, String, ScrolledWindow, SourceView)> =
            HashMap::new();

        let plus_tab = gtk::Box::new(Orientation::Horizontal, 0);
        let btn = gtk::Button::new();
        btn.set_label("+");
        plus_tab.pack_start(&btn, false, false, 0);
        plus_tab.show_all();

        connect!(relm, btn, connect_clicked(_), Msg::RequestingNewEnvironment);

        let plus_box = gtk::Box::new(Orientation::Horizontal, 0);
        plus_box.show();

        let entry_tab = gtk::Box::new(Orientation::Horizontal, 0);
        let entry = gtk::Entry::new();
        entry_tab.pack_start(&entry, false, false, 0);
        entry_tab.show_all();
        let entry_box = gtk::Box::new(Orientation::Horizontal, 0);
        entry_box.show();

        connect!(
            relm,
            entry,
            connect_key_press_event(_, key),
            return (Msg::NewEntryPressingKey(key.clone()), Inhibit(false),)
        );

        main_box.pack_start(&notebook, true, true, 5);
        main_box.set_margin_top(5);
        main_box.set_margin_bottom(5);
        main_box.show_all();

        connect!(
            relm,
            notebook,
            connect_switch_page(_, _, id),
            Msg::TogglingEnvironmentIndex(id)
        );

        relm.stream().emit(Msg::CreatingNewTabPageButton);
        EnvironEditor {
            main_box,
            notebook,
            environ_sources,
            entry,
            model,
            plus_tab: (plus_tab, plus_box),
            entry_tab: (entry_tab, entry_box),
            relm: relm.clone(),
        }
    }
}
