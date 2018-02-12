use std::vec::Vec;

use serde_json;

use super::super::helpers::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    verb: String,
    url: String,
    headers: Vec<String>,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    name: String,
    template: Template,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    name: String,
    queries: Vec<Query>,
}

#[derive(Debug)]
pub struct Workspace {
    filename: String,
    payload: Payload,
}

impl Workspace {
    pub fn new() -> Self {
        Workspace {
            filename: "rustaman.json".to_owned(),
            payload: Payload {
                name: "Rustaman".to_owned(),
                queries: vec![],
            },
        }
    }

    pub fn from_file(filename: &str) -> Result<Self, path::IOError> {
        let cfg = path::read_file(filename)?;
        let payload = serde_json::from_str::<Payload>(cfg.as_str()).unwrap(); // crash if the format
        let workspace = Workspace {
            filename: filename.to_owned(),
            payload: payload,
        };
        Ok(workspace)
    }

    pub fn default() -> Self {
        match Workspace::from_file("rustaman.json") {
            Ok(res) => res,
            Err(_) => {
                let workspace = Workspace::new();
                workspace.sync().unwrap();
                workspace
            }
        }
    }

    pub fn sync(&self) -> Result<(), path::IOError> {
        let filepath = path::workspace(self.filename())?;
        let filepath = filepath
            .to_str()
            .expect("Cannot create a filepath for the current workspace");
        let filecontent = serde_json::to_string_pretty(&self.payload);
        let filecontent =
            filecontent.expect("Unable to save workspace, cannot serilizing it to json");
        path::write_file(filepath, filecontent.as_str())?;
        Ok(())
    }

    fn filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn name(&self) -> &str {
        self.payload.name.as_str()
    }
}
