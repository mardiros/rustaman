// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use serde_json;
use sourceview5::{self, prelude::*};

use crate::helpers::sourceview::create_buffer;

fn prettify_js(payload: &str) -> Result<String, serde_json::Error> {
    let obj: serde_json::Value = serde_json::from_str(payload)?;
    Ok(serde_json::to_string_pretty(&obj).unwrap())
}

#[derive(Debug, Clone)]
pub enum ResponseBodyMsg {}

pub struct ResponseBody {}

pub struct Widgets {}

impl SimpleComponent for ResponseBody {
    type Init = ();
    type Input = ResponseBodyMsg;
    type Output = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        _request: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = create_buffer("json");
        let response_view = sourceview5::View::with_buffer(&buffer);
        response_view.set_margin_all(10);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    response_view -> SourceView {
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                }
            }
        }

        ComponentParts {
            model: ResponseBody {},
            widgets: Widgets {},
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
