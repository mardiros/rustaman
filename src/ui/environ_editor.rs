use std::collections::HashMap;
use std::vec::Vec;

use gdk;
use gdk::enums::key;
use gtk::{self, IconSize, Orientation, ReliefStyle};
use gtk::prelude::*;
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView};
use sourceview::prelude::*;
use relm::{Relm, Update, Widget};
use handlebars::Handlebars;
use serde_yaml;

use super::super::models::Environment;

pub struct Model {
    current: usize,
    environments: Vec<Environment>,
}

impl Model {
    pub fn environments(&self) -> &[Environment] {
        self.environments.as_slice()
    }

    pub fn set_environment_payload(&mut self, id: usize, payload: &str) {
        for (idx, env) in self.environments.iter_mut().enumerate() {
            if idx == id {
                env.set_payload(payload);
                break;
            }
        }
    }

    pub fn push_environment(&mut self, environment: &Environment) {
        self.environments.push(environment.clone());
    }

    pub fn delete_environment(&mut self, id: usize) {
        self.environments.remove(id);
    }
}

#[derive(Msg)]
pub enum Msg {
    CompilingTemplate(String),
    TemplateCompiled(String),
    TemplateCompilationFailed(String),
    Saving(usize, Environment),
    NewEntryPressingKey(gdk::EventKey),
    RequestingNewEnvironment,
    CreatingNewTabPageButton,
    CreatingEnvironment(String),
    AppendingEnvironment(Environment),
    EnvironmentCreated(Environment),
    TogglingEnvironment(u32),
}

pub struct EnvironEditor {
    notebook: gtk::Notebook,
    environ_sources: HashMap<u32, (String, SourceView)>,
    relm: Relm<EnvironEditor>,
    plus_tab: (gtk::Box, gtk::Box),
    entry_tab: (gtk::Box, gtk::Box),
    entry: gtk::Entry,
    model: Model,
}

impl EnvironEditor {
    fn get_text(&self) -> Option<String> {
        let current = self.model.current as u32;
        info!("{:?}", self.environ_sources);
        let &(_, ref environ_source) = self.environ_sources.get(&current).unwrap();

        let buffer = environ_source.get_buffer().unwrap();
        let start_iter = buffer.get_start_iter();
        let end_iter = buffer.get_end_iter();
        buffer.get_text(&start_iter, &end_iter, true)
    }
}

impl Update for EnvironEditor {
    type Model = Model;
    type ModelParam = Vec<Environment>;
    type Msg = Msg;

    fn model(_: &Relm<Self>, environments: Vec<Environment>) -> Model {
        Model {
            current: 0,
            environments: environments,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CompilingTemplate(template) => {
                let payload = match self.get_text() {
                    Some(data) => data,
                    None => "".to_owned(),
                };

                let mut reg = Handlebars::new();
                debug!("Template: {}", template.as_str());
                debug!("Params: {}", payload.as_str());
                let params: serde_yaml::Value = serde_yaml::from_str(&payload).unwrap();
                let res = reg.render_template(template.as_str(), &params);
                match res {
                    Ok(rendered) => {
                        debug!("Rendered: {}", rendered);
                        let id = self.model.current;
                        self.model.set_environment_payload(id, payload.as_str());
                        self.relm.stream().emit(Msg::TemplateCompiled(rendered));
                        self.relm
                            .stream()
                            .emit(Msg::Saving(id, self.model.environments()[id].to_owned()));
                    }
                    Err(err) => {
                        let err = format!("{:?}", err);
                        self.relm
                            .stream()
                            .emit(Msg::TemplateCompilationFailed(err.to_owned()));
                    }
                }
            }
            Msg::RequestingNewEnvironment => {
                info!("Detach mentPlus");
                self.notebook.detach_tab(&self.plus_tab.1);
                info!("Attach Entry");
                let _index = self.notebook
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
                        let _index = self.notebook
                            .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
                    }
                    _ => {}
                }
            }
            Msg::AppendingEnvironment(env) => {
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

                    tab.pack_start(&label, false, false, 0);
                    tab.pack_start(&button, false, false, 0);
                    tab.show_all();
                    tab
                };

                let tab_page = {
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
                    environ_source
                };

                connect!(
                    self.relm,
                    tab_page,
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
                    .insert(index, (name.to_owned(), tab_page));

                self.model.push_environment(&env);
            }

            Msg::EnvironmentCreated(env) => {
                info!("Detach Add new tab");
                self.notebook.detach_tab(&self.entry_tab.0);
                self.update(Msg::AppendingEnvironment(env));
                info!("Attach Add new tab");
                let _index = self.notebook
                    .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
            }

            Msg::TogglingEnvironment(id) => {
                info!("Switch to page {}", id);
                {
                    let env = self.model.environments();
                    if self.model.current < env.len() {
                        self.relm.stream().emit(Msg::Saving(self.model.current, {
                            env[self.model.current].to_owned()
                        }));
                    }
                }
                self.model.current = id as usize;
            }
            Msg::CreatingNewTabPageButton => {
                let _index = self.notebook
                    .append_page(&self.plus_tab.1, Some(&self.plus_tab.0));
            }
            _ => {}
        }
    }
}

impl Widget for EnvironEditor {
    type Root = gtk::Notebook;

    fn root(&self) -> Self::Root {
        self.notebook.clone()
    }

    fn init_view(&mut self) {}

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating Environ widget");

        let notebook = gtk::Notebook::new();
        notebook.set_hexpand(false);
        notebook.set_vexpand(true);
        notebook.set_margin_top(10);
        notebook.set_margin_left(10);

        if model.environments().len() > 0 {
            for env in model.environments() {
                relm.stream().emit(Msg::AppendingEnvironment(env.clone()));
            }
        }
        let environ_sources: HashMap<u32, (String, SourceView)> = HashMap::new();

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

        notebook.set_hexpand(true);
        notebook.set_vexpand(false);
        notebook.set_size_request(800, 480);
        notebook.show();

        connect!(
            relm,
            notebook,
            connect_switch_page(_, _, id),
            Msg::TogglingEnvironment(id)
        );

        relm.stream().emit(Msg::CreatingNewTabPageButton);
        EnvironEditor {
            relm: relm.clone(),
            notebook: notebook,
            environ_sources: environ_sources,
            plus_tab: (plus_tab, plus_box),
            entry_tab: (entry_tab, entry_box),
            entry: entry,
            model: model,
        }
    }
}
