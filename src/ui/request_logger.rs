use glib::GString;
use gtk::{self, Orientation, ScrolledWindow};
use relm::{Relm, Update, Widget};
use sourceview::{self, prelude::*, LanguageManager, StyleSchemeManager, View as SourceView};

use super::super::helpers::http::HttpRequest;

#[derive(Msg)]
pub enum Msg {
    ExecutingRequest(HttpRequest),
    RequestExecuted(String),
}

pub struct RequestLogger {
    hbox: gtk::Box,
    logger_view: SourceView,
    //relm: Relm<RequestLogger>,
}

impl RequestLogger {
    fn append_text(&mut self, text: &str) {
        let buffer = self.logger_view.get_buffer().unwrap();
        let start_iter = buffer.get_start_iter();
        let end_iter = buffer.get_end_iter();
        let current = match buffer.get_text(&start_iter, &end_iter, true) {
            Some(data) => data,
            None => GString::from(""),
        };
        let mut current = current.as_str().to_string();
        current.push_str(text);
        buffer.set_text(current.as_str());
    }
}

impl Update for RequestLogger {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ExecutingRequest(request) => {
                let mut text = String::from("\n>>> New request\n");
                let authority = request.authority();
                let authority = format!("#! Authority: {}:{}\n\n", authority.0, authority.1);
                text.push_str(authority.as_str());
                text.push_str(request.http_frame());
                self.append_text(text.as_str());
                self.append_text(">>> End of request\n");
            }

            Msg::RequestExecuted(response) => {
                let mut text = String::from("<<< Response\n");
                text.push_str(response.as_str());
                self.append_text(text.as_str());
                self.append_text("\n<<< End of response\n");
            }
        }
    }
}

impl Widget for RequestLogger {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        info!("Creating RequestLogger widget");
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);

        let langmngr = LanguageManager::get_default().unwrap();
        let lang = langmngr.get_language("rustaman-response").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        let style = stylemngr.get_scheme("rustaman-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(Some(&style));

        let logger_view = SourceView::new_with_buffer(&buffer);
        logger_view.set_hexpand(true);
        logger_view.set_vexpand(true);

        let scrollview = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        scrollview.add(&logger_view);

        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);
        hbox.pack_start(&scrollview, true, true, 5);

        hbox.show_all();
        RequestLogger {
            hbox,
            logger_view,
            //relm: relm.clone(),
        }
    }
}
