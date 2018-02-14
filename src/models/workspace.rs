use std::vec::Vec;

use serde_json;

use super::super::helpers::path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template {
    verb: String,
    url: String,
    headers: Vec<String>,
    body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query {
    name: String,
    template: Template,
}

pub type Queries = Vec<Query>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    name: String,
    queries: Queries,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    filepath: String,
    payload: Payload,
}

impl Workspace {
    pub fn new(filepath: &str) -> Self {
        Workspace {
            filepath: filepath.to_owned(),
            payload: Payload {
                name: "Rustaman".to_owned(),
                queries: vec![],
            },
        }
    }

    pub fn from_file(filepath: &str) -> Result<Self, path::IOError> {
        info!("Try loading workspace from file {}", filepath);
        let cfg = path::read_file(filepath)?;
        debug!("File {} readed ({} chars.)", filepath, cfg.len());
        let payload = serde_json::from_str::<Payload>(cfg.as_str()).unwrap(); // crash if the format
        let workspace = Workspace {
            filepath: filepath.to_owned(),
            payload: payload,
        };
        info!("Workspace loaded from file {}", filepath);
        Ok(workspace)
    }

    pub fn default() -> Self {
        let filepath = path::workspace("rustaman.json").expect("Cannot build default workspace filename");
        match Workspace::from_file(filepath.to_str().unwrap()) {
            Ok(res) => res,
            Err(err) => {
                error!("Error while loading workspace from filem creating default: {}", err);
                let workspace = Workspace::new(filepath.to_str().unwrap());
                workspace.sync().unwrap();
                info!("New workspace created");
                workspace
            }
        }
    }

    pub fn sync(&self) -> Result<(), path::IOError> {
        info!("Wrting workspace in file {}", self.filepath);
        let filecontent = serde_json::to_string_pretty(&self.payload);
        let filecontent =
            filecontent.expect("Unable to save workspace, cannot serilizing it to json");
        path::write_file(self.filepath.as_str(), filecontent.as_str())?;
        Ok(())
    }

    fn filepath(&self) -> &str {
        self.filepath.as_str()
    }

    pub fn name(&self) -> &str {
        self.payload.name.as_str()
    }

    pub fn queries(&self) -> &[Query] {
        self.payload.queries.as_slice()
    }
}
