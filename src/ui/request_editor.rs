//use gdk;
//use gdk::enums::key;
use gtk::{self, Orientation};
use gtk::prelude::*;
use sourceview::View as SourceView;
use relm::{Relm, Update, Widget};

use super::super::models::Template;

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
}

pub struct RequestEditor {
    hbox: gtk::Box,
    request_source: SourceView,
    relm: Relm<RequestEditor>,
    model: Model,
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
                let buffer = self.request_source.get_buffer().unwrap();
                let start_iter = buffer.get_start_iter();
                let end_iter = buffer.get_end_iter();
                let text = buffer.get_text(&start_iter, &end_iter, true);
                match text {
                    Some(ref data) => {
                        self.relm.stream().emit(Msg::Save(id, data.to_owned()));
                    }
                    None => {
                        error!("No data to save");
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
        let request_source = SourceView::new();
        request_source.set_margin_left(10);
        request_source.set_hexpand(true);
        request_source.set_vexpand(true);
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
