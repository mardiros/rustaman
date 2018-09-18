use gdk;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{self, Orientation, ScrolledWindow};
use relm::{Relm, Update, Widget};
use sourceview::prelude::*;
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView};

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
    RequestingSave(usize),
    Saving(usize, Template),
    TemplateChanged(Template),
    ExecutingCurrent,
    Executing(Template),
    Hiding,
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
                self.hbox.show_all();
            }
            Msg::RequestingSave(id) => {
                info!("Save Template request");
                let text = self.get_text();
                match text {
                    Some(ref data) => {
                        self.relm.stream().emit(Msg::Saving(id, data.to_owned()));
                    }
                    None => {
                        error!("No data to save");
                    }
                }
            }
            Msg::ExecutingCurrent => {
                let text = self.get_text();
                match text {
                    Some(ref data) => {
                        error!("Running query");
                        self.relm.stream().emit(Msg::Executing(data.to_owned()));
                    }
                    None => {
                        error!("No requests to execute");
                        // Alert something here
                    }
                }
            }
            Msg::Hiding => {
                self.hbox.hide();
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
        let lang = langmngr.get_language("rustaman-json").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        let style = stylemngr.get_scheme("solarized-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(&style);

        let request_source = SourceView::new_with_buffer(&buffer);
        request_source.set_insert_spaces_instead_of_tabs(true);
        request_source.set_tab_width(2);
        request_source.set_show_line_numbers(true);
        request_source.set_highlight_current_line(true);
        request_source.set_monospace(true);

        let scrollview = ScrolledWindow::new(None, None);
        scrollview.add(&request_source);

        connect!(
            relm,
            request_source,
            connect_key_press_event(_, key_),
            return Inhibit(
                key_.get_state().intersects(gdk::ModifierType::CONTROL_MASK)
                    && match key_.get_keyval() {
                        key::Return => true,
                        _ => false,
                    }
            )
        );

        hbox.pack_start(&scrollview, true, true, 5);
        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);

        RequestEditor {
            hbox,
            request_source,
            model,
            relm: relm.clone(),
        }
    }
}
