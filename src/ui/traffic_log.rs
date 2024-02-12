// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};
use sourceview5;

use crate::helpers::sourceview::create_buffer;

#[derive(Debug, Clone)]
pub enum TrafficLogMsg {}

pub struct TrafficLog {}

pub struct Widgets {}

impl SimpleComponent for TrafficLog {
    type Init = ();
    type Input = TrafficLogMsg;
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
        let buffer = create_buffer("rustaman-response");
        let request_source = sourceview5::View::with_buffer(&buffer);
        request_source.set_margin_all(10);

        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_spacing: 5,
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[local_ref]
                    request_source -> SourceView {
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                }
            }
        }

        ComponentParts {
            model: TrafficLog {},
            widgets: Widgets {},
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
