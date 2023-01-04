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

use std::io::Write;
use std::vec::Vec;

use clap::Parser;
use clap::ValueHint::FilePath;

use relm::Widget;
use sourceview::{prelude::*, LanguageManager, StyleSchemeManager};

use crate::errors::{RustamanError, RustamanResult};
use crate::ui::window::Window;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Use specific workspace file expect a file like: workspace.json
    #[clap(short='w', long="workspace",name="WORKSPACE", value_hint=FilePath)]
    workspace: Option<String>,
}

fn run() -> RustamanResult<()> {

    let args = Args::parse();
    gtk::init().expect("Unable to initialize gtk");

    let workspace = args.workspace;
    let workspace = if let Some(filepath) = workspace {
        models::Workspace::from_file(&filepath)
            .or_else(|_| -> RustamanResult<models::Workspace> {
                let workspace = models::Workspace::new(&filepath);
                workspace.sync()?;
                Ok(workspace)
            })
            .unwrap()
    } else {
        models::Workspace::default()
    };

    let conf_path = helpers::path::rustaman_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let langmngr = LanguageManager::get_default().ok_or(RustamanError::GtkStrError(
        "Unable to load the Language Manager".to_string(),
    ))?;
    let mut search_path = langmngr.get_search_path();
    search_path.push(glib::GString::from(helpers::path::assets_dir()));
    search_path.push(glib::GString::from(conf_path.clone()));
    let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
    info!("Set langmngr search path: {:?}", path2);
    langmngr.set_search_path(path2.as_slice());

    let stylemngr = StyleSchemeManager::get_default().ok_or(RustamanError::GtkStrError(
        "Unable to load the Style Scheme Manager".to_string(),
    ))?;
    let mut style_path = stylemngr.get_search_path();
    style_path.push(glib::GString::from(helpers::path::assets_dir()));
    style_path.push(glib::GString::from(conf_path));
    let path2: Vec<&str> = style_path.iter().map(|path| path.as_str()).collect();
    info!("Set search path: {:?}", path2);
    stylemngr.set_search_path(path2.as_slice());

    Window::run(workspace).map_err(|_| {
        RustamanError::GtkStrError("Unexpected error while running the window".to_string())
    })?;
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    info!("Starting Rustaman");
    match run() {
        Ok(()) => {
            debug!("Rustaman ended succesfully");
        }
        Err(err) => {
            let _ = writeln!(&mut std::io::stderr(), "{}", err);
            std::process::exit(1);
        }
    }
}
