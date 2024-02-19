// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use std::time::Duration;

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};

#[derive(Debug, Clone)]
pub enum StatusLineMsg {
    ReceivingHttpResponse(String, Duration),
}

fn build_markup_for_status(status: &str) -> String {
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

fn build_markup_for_elapsed(elapsed: u64) -> String {
    let color = if elapsed < 700 { "#088A29" } else { "#B40404" };
    let fmt = format!(
        r#"<span face="monospace" background="{}" size="large"> {}ms </span>"#,
        color, elapsed
    );
    fmt
}

pub struct StatusLine {
    status_line: String,
    elapsed: Option<Duration>,
}

impl StatusLine {}

impl StatusLine {}

pub struct Widgets {
    status_line: gtk::Label,
    elapsed: gtk::Label,
}

impl Component for StatusLine {
    type Init = ();
    type Input = StatusLineMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::new(gtk::Orientation::Horizontal, 5)
    }

    fn init(
        _request: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let status_line = gtk::Label::new(None);
        let elapsed = gtk::Label::new(None);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_margin_start: 5,
                #[local_ref]
                status_line -> gtk::Label{
                    set_margin_start: 5,
                },
                #[local_ref]
                elapsed -> gtk::Label{
                    set_margin_start: 5,
                },
            }
        }

        ComponentParts {
            model: StatusLine {
                status_line: "".to_string(),
                elapsed: None,
            },
            widgets: Widgets {
                status_line,
                elapsed,
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            StatusLineMsg::ReceivingHttpResponse(response, elapsed) => {
                let first_line = response.lines().next().unwrap_or("").to_string();
                let v: Vec<&str> = first_line.splitn(2, ' ').collect();
                self.status_line = v.last().unwrap_or(&"").to_string();
                self.elapsed = Some(elapsed)
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets
            .status_line
            .set_markup(build_markup_for_status(self.status_line.as_str()).as_str());
        if let Some(elapsed) = self.elapsed {
            let sec = elapsed.as_secs();
            let ms = (elapsed.subsec_millis() as u64) + sec * 1000;
            widgets
                .elapsed
                .set_markup(build_markup_for_elapsed(ms).as_str());
        }
    }
}
