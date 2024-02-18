#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::io::Write;

use clap::Parser;

use relm4::adw;
use relm4::gtk;
use relm4::prelude::*;
use relm4_icons;
use sourceview5::{LanguageManager, StyleSchemeManager};

mod errors;
mod helpers;
mod models;
mod ui;

use crate::errors::RustamanResult;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// filepath to the workspace to load
    #[arg(short, long)]
    workspace: Option<String>,
}

fn init_gtk() -> RustamanResult<()> {
    let conf_path = helpers::path::rustaman_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    gtk::init().expect("Unable to initialize gtk");

    let sm = adw::StyleManager::default();
    sm.set_color_scheme(adw::ColorScheme::ForceDark);

    let langmngr = LanguageManager::default();
    let mut search_path = langmngr.search_path();
    search_path.push(helpers::path::assets_dir().into());
    search_path.push(conf_path.clone().into());
    let current_assets_path = env::current_dir()?.join("assets");
    debug!("Adding {:?}", current_assets_path.to_str());
    search_path.push(current_assets_path.to_str().unwrap().to_string().into());

    let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
    // info!("Set langmngr search path: {:?}", path2);
    langmngr.set_search_path(path2.as_slice());

    let stylemngr = StyleSchemeManager::default();
    let mut style_path = stylemngr.search_path();
    style_path.push(helpers::path::assets_dir().into());
    style_path.push(conf_path.into());
    style_path.push(current_assets_path.to_str().unwrap().to_string().into());
    let path2: Vec<&str> = style_path.iter().map(|path| path.as_str()).collect();
    info!("Set style_path search path: {:?}", path2);
    stylemngr.set_search_path(path2.as_slice());

    relm4_icons::initialize_icons();
    Ok(())
}

fn run() -> RustamanResult<()> {
    let args = Args::parse();
    init_gtk()?;

    let workspace = if let Some(filepath) = args.workspace {
        models::Workspace::from_file(filepath.as_str())
            .or_else(|_| -> RustamanResult<models::Workspace> {
                let workspace = models::Workspace::new(filepath.as_str());
                workspace.sync()?;
                Ok(workspace)
            })
            .unwrap()
    } else {
        models::Workspace::default()
    };
    let app: RelmApp<ui::window::AppMsg> = RelmApp::new("relm4.rustaman");
    app.with_args(Vec::new()).run::<ui::window::App>(workspace);

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
