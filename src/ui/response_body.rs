// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use serde_json;
use sourceview5::{self, prelude::*};

fn prettify_js(payload: &str) -> Result<String, serde_json::Error> {
    let obj: serde_json::Value = serde_json::from_str(payload)?;
    Ok(serde_json::to_string_pretty(&obj).unwrap())
}

#[derive(Debug, Clone)]
pub enum ResponseBodyMsg {}

pub struct ResponseBody {}

pub struct Widgets {}

impl Component for ResponseBody {
    type Init = ();
    type Input = ResponseBodyMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        request: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_highlight_syntax(true);

        let langmngr = sourceview5::LanguageManager::default();
        let stmngr = sourceview5::StyleSchemeManager::default();

        let search_path = langmngr.search_path();
        debug!("{:?}", search_path);

        if let Some(ref language) = langmngr.language("rustaman-response") {
            buffer.set_language(Some(language));
        } else {
            error!(
                "Can't find rustaman-response.lang lang in {:?}",
                search_path
            )
        }
        if let Some(ref scheme) = stmngr.scheme("rustaman-dark") {
            buffer.set_style_scheme(Some(scheme));
        } else {
            error!("Can't find rustaman-dark.xml theme   in {:?}", search_path)
        }

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

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {}

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
