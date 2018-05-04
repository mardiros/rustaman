use gtk::prelude::*;
use gtk::{self, Orientation};
use relm::{Relm, Update, Widget};
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView, prelude::*};

#[derive(Msg)]
pub enum Msg {
    RequestExecuted(String),
}

pub struct Response {
    hbox: gtk::Box,
    response_view: SourceView,
    //relm: Relm<Response>,
}

impl Update for Response {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::RequestExecuted(response) => {
                let buffer = self.response_view.get_buffer().unwrap();
                buffer.set_text(response.as_str());
            }
        }
    }
}

impl Widget for Response {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        info!("Creating Response widget");
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);

        let langmngr = LanguageManager::get_default().unwrap();
        let lang = langmngr.get_language("rustaman-response").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        let style = stylemngr.get_scheme("solarized-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(&style);

        let response_view = SourceView::new_with_buffer(&buffer);

        response_view.set_editable(false);
        response_view.set_hexpand(true);
        response_view.set_vexpand(true);
        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);
        hbox.pack_start(&response_view, true, true, 5);

        hbox.show_all();
        Response {
            hbox: hbox,
            response_view: response_view,
            //relm: relm.clone(),
        }
    }
}
