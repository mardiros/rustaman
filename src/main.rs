#![feature(proc_macro)]

extern crate gdk;
extern crate gtk;

#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod ui;
mod models;
mod helpers;

use relm::Widget;
use ui::window::Window;

fn main() {
    let workspace = models::Workspace::default();
    Window::run(workspace).unwrap();
}
