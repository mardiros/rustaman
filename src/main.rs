#![feature(proc_macro)]

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate sourceview;

#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate cabot;
extern crate regex;

mod ui;
mod models;
mod helpers;

use relm::Widget;
use ui::window::Window;

fn main() {
    pretty_env_logger::init();
    info!("Starting Rustaman");
    let workspace = models::Workspace::default();
    Window::run(workspace).unwrap();
}
