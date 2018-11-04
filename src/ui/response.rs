use gtk::prelude::*;
use gtk::{self, Orientation, ScrolledWindow};
use relm::{Relm, Update, Widget};
use serde_json;
use sourceview::{self, prelude::*, LanguageManager, StyleSchemeManager, View as SourceView};

fn prettify_js(payload: &str) -> String {
    let obj: serde_json::Value = serde_json::from_str(payload).unwrap();
    serde_json::to_string_pretty(&obj).unwrap()
}

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
                let mut is_json = false;
                let mut has_content = true;
                let mut text = String::new();
                let mut lines = response.lines();
                loop {
                    let line = lines.next();
                    match line {
                        Some(unwrapped) => {
                            if unwrapped.is_empty() {
                                break;
                            }
                            if unwrapped.starts_with("Content-Type: application/json") {
                                is_json = true;
                            }
                        }
                        None => has_content = false,
                    }
                }
                if has_content {
                    loop {
                        let line = lines.next();
                        match line {
                            Some(unwrapped) => {
                                text.push_str(unwrapped);
                                text.push('\n');
                            }
                            None => break,
                        }
                    }
                }
                let response = if is_json && has_content {
                    prettify_js(text.as_str())
                } else {
                    text
                };
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
        response_view.set_hexpand(true);
        response_view.set_vexpand(true);
        response_view.set_editable(false);

        let scrollview = ScrolledWindow::new(None, None);
        scrollview.add(&response_view);

        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);
        hbox.pack_start(&scrollview, true, true, 5);

        hbox.show_all();
        Response {
            hbox,
            response_view,
            //relm: relm.clone(),
        }
    }
}
