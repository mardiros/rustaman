#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::io::Write;

use clap::Parser;



mod errors;
mod helpers;
mod models;

use crate::errors::RustamanResult;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    wokspace: Option<String>,

}

fn run() -> RustamanResult<()> {
    let args = Args::parse();

    let workspace = if let Some(filepath) = args.wokspace {
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
