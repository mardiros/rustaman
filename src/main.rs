#[macro_use]
extern crate log;

#[macro_use]
extern crate relm_derive;

#[macro_use]
extern crate serde_derive;

mod errors;
mod helpers;
mod models;
mod ui;

use std::vec::Vec;

use relm::Widget;
use sourceview::{prelude::*, LanguageManager, StyleSchemeManager};

use crate::ui::window::Window;


fn main() {
    pretty_env_logger::init();
    info!("Starting Rustaman");
    gtk::init().expect("Unable to initialize gtk");

    let conf_path = helpers::path::rustaman_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let langmngr = LanguageManager::get_default().unwrap();
    let mut search_path = langmngr.get_search_path();
    search_path.push(conf_path.clone());
    let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
    info!("Set langmngr search path: {:?}", path2);
    langmngr.set_search_path(path2.as_slice());

    let stylemngr = StyleSchemeManager::get_default().unwrap();
    let mut style_path = stylemngr.get_search_path();
    style_path.push(conf_path);
    let path2: Vec<&str> = style_path.iter().map(|path| path.as_str()).collect();
    info!("Set search path: {:?}", path2);
    stylemngr.set_search_path(path2.as_slice());
    
    let workspace = models::Workspace::default();
    Window::run(workspace).unwrap();
}
