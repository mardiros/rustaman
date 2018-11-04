use std::time::SystemTime;

use gtk::prelude::*;
use gtk::{self, Label, Orientation};
use relm::{Relm, Update, Widget};

use super::super::helpers::http::HttpRequest;

pub struct Model {
    time: SystemTime,
}

#[derive(Msg)]
pub enum Msg {
    ExecutingRequest(HttpRequest),
    RequestExecuted(String),
}

pub struct ResponseStatus {
    model: Model,
    hbox: gtk::Box,
    status_label: Label,
    time_label: Label,
    //relm: Relm<Response>,
}

impl ResponseStatus {
    fn build_markup_for_status(&self, status: &str) -> String {
        let color = match status.chars().next() {
            Some('2') => "#088A29",
            Some('3') => "#088A29",
            Some('4') => "#FE642E",
            _ => "#B40404",
        };
        let fmt = format!(
            r#"<span face="monospace" background="{}" size="large"> {} </span>"#,
            color, status
        );
        fmt
    }

    fn build_markup_for_elapsed(&self, elapsed: u64) -> String {
        let color = if elapsed < 700 { "#088A29" } else { "#B40404" };
        let fmt = format!(
            r#"<span face="monospace" background="{}" size="large"> {}ms </span>"#,
            color, elapsed
        );
        fmt
    }
}

impl Update for ResponseStatus {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            time: SystemTime::now(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ExecutingRequest(request) => {
                self.model.time = SystemTime::now();
                let req = request.http_frame().lines().next();
                match req {
                    Some(r) => self.status_label.set_markup(r),
                    None => self.status_label.set_markup(r#"No request found"#),
                }
            }
            Msg::RequestExecuted(response) => {
                let duration = self.model.time.elapsed().unwrap(); // SystemTimeError!
                let sec = duration.as_secs();
                let ms = (duration.subsec_millis() as u64) + sec * 1000;
                let markup = self.build_markup_for_elapsed(ms);
                self.time_label.set_markup(markup.as_str());
                let resp = response.lines().next();
                match resp {
                    Some(r) => {
                        let v: Vec<&str> = r.splitn(2, ' ').collect();
                        if let Some(status) = v.last() {
                            let markup = self.build_markup_for_status(status);
                            self.status_label.set_markup(markup.as_str());
                        }
                    }
                    None => self.status_label.set_markup(r#"No response found"#),
                }
            }
        }
    }
}

impl Widget for ResponseStatus {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(_relm: &Relm<Self>, model: Model) -> Self {
        info!("Creating ResponseStatus widget");
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        let status_label = gtk::Label::new(None);
        let time_label = gtk::Label::new(None);

        hbox.pack_start(&status_label, false, false, 10);
        hbox.pack_start(&time_label, false, false, 10);
        hbox.show_all();

        ResponseStatus {
            model,
            hbox,
            status_label,
            time_label,
        }
    }
}
