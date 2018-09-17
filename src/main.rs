#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate gdk;
extern crate gtk;
extern crate sourceview;

#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

extern crate cabot;
extern crate regex;

extern crate handlebars;

mod helpers;
mod models;
mod ui;

use std::vec::Vec;

use relm::Widget;
use sourceview::prelude::*;
use ui::window::Window;

fn main() {
    pretty_env_logger::init();
    info!("Starting Rustaman");
    gtk::init();

    let langmngr = sourceview::LanguageManager::get_default().unwrap();
    let mut search_path = langmngr.get_search_path();
    let conf_path = helpers::path::config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    search_path.push(conf_path);
    let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
    info!("Set search path: {:?}", path2);
    langmngr.set_search_path(path2.as_slice());

    let workspace = models::Workspace::default();
    Window::run(workspace).unwrap();
}
