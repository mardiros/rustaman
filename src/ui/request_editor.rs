use std::vec::Vec;

use gdk;
use gdk::enums::key;
use gtk::{self, Orientation};
use gtk::prelude::*;
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView};
use sourceview::prelude::*;
use relm::{Relm, Update, Widget};

use super::super::models::Template;
use super::super::helpers::path;

pub struct Model {
    template: Template,
}

impl Model {
    pub fn set_template(&mut self, template: &str) {
        self.template = template.to_owned();
    }
}

#[derive(Msg)]
pub enum Msg {
    SaveRequest(usize),
    Save(usize, Template),
    TemplateChanged(Template),
    RequestSourceKeyPress(gdk::EventKey),
    Execute(Template),
}

pub struct RequestEditor {
    hbox: gtk::Box,
    request_source: SourceView,
    relm: Relm<RequestEditor>,
    model: Model,
}

impl RequestEditor {
    fn get_text(&self) -> Option<String> {
        let buffer = self.request_source.get_buffer().unwrap();
        let start_iter = buffer.get_start_iter();
        let end_iter = buffer.get_end_iter();
        buffer.get_text(&start_iter, &end_iter, true)
    }
}

impl Update for RequestEditor {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _request: ()) -> Model {
        Model {
            template: "".to_owned(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::TemplateChanged(template) => {
                info!("Template changed: {}", template);
                self.model.set_template(template.as_str());
                let buffer = self.request_source.get_buffer().unwrap();
                buffer.set_text(template.as_str());
            }
            Msg::SaveRequest(id) => {
                info!("Save Template request");
                let text = self.get_text();
                match text {
                    Some(ref data) => {
                        self.relm.stream().emit(Msg::Save(id, data.to_owned()));
                    }
                    None => {
                        error!("No data to save");
                    }
                }
            }
            Msg::RequestSourceKeyPress(key) => {
                let keystate = key.get_state();
                if keystate.intersects(gdk::ModifierType::CONTROL_MASK) {
                    let keyval = key.get_keyval();
                    match keyval {
                        key::Return => {
                            let text = self.get_text();
                            match text {
                                Some(ref data) => {
                                    error!("Running query");
                                    self.relm.stream().emit(Msg::Execute(data.to_owned()));
                                }
                                None => {
                                    error!("No requests to execute");
                                    // Alert something here
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl Widget for RequestEditor {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating RequestView widget {:?}", model.template);
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);

        let langmngr = LanguageManager::get_default().unwrap();
        let mut search_path = langmngr.get_search_path();
        search_path.push(path::config_dir().unwrap().to_str().unwrap().to_owned());
        let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
        langmngr.set_search_path(path2.as_slice());
        let lang = langmngr.get_language("rustaman-json").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        println!("{:?}", stylemngr.get_search_path());
        println!("{:?}", stylemngr.get_scheme_ids());
        let style = stylemngr.get_scheme("solarized-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(&style);

        let request_source = SourceView::new_with_buffer(&buffer);
        request_source.set_margin_left(10);
        request_source.set_hexpand(true);
        request_source.set_vexpand(true);
        request_source.set_insert_spaces_instead_of_tabs(true);
        request_source.set_tab_width(2);
        request_source.set_show_line_numbers(true);
        request_source.set_highlight_current_line(true);
        request_source.set_monospace(true);

        connect!(
            relm,
            request_source,
            connect_key_press_event(_, key_),
            return (
                Msg::RequestSourceKeyPress(key_.clone()),
                Inhibit(
                    key_.get_state().intersects(gdk::ModifierType::CONTROL_MASK)
                        && match key_.get_keyval() {
                            key::Return => true,
                            _ => false,
                        }
                )
            )
        );

        hbox.add(&request_source);

        hbox.show_all();
        RequestEditor {
            hbox: hbox,
            request_source: request_source,
            relm: relm.clone(),
            model: model,
        }
    }
}
