use gtk::{self, Orientation};
use gtk::prelude::*;
use sourceview::{self, View as SourceView, StyleSchemeManager, LanguageManager};
use sourceview::prelude::*;
use relm::{Relm, Update, Widget};

use super::super::helpers::path;

#[derive(Msg)]
pub enum Msg {
}

pub struct EnvironEditor {
    hbox: gtk::Box,
    environ_view: SourceView,
    //relm: Relm<EnvironEditor>,
}

impl Update for EnvironEditor {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {}
    }
}

impl Widget for EnvironEditor {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.hbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        info!("Creating Environ widget");
        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);

        let langmngr = LanguageManager::get_default().unwrap();
        let mut search_path = langmngr.get_search_path();
        search_path.push(path::config_dir().unwrap().to_str().unwrap().to_owned());
        let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
        langmngr.set_search_path(path2.as_slice());
        let lang = langmngr.get_language("rustaman-environ").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        let style = stylemngr.get_scheme("solarized-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(&style);

        let environ_view = SourceView::new_with_buffer(&buffer);

        environ_view.set_margin_left(10);
        environ_view.set_hexpand(true);
        environ_view.set_vexpand(true);
        hbox.add(&environ_view);
        hbox.show_all();
        EnvironEditor {
            hbox: hbox,
            environ_view: environ_view,
            //relm: relm.clone(),
        }
    }
}
