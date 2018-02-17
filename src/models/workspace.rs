use std::vec::Vec;

use serde_json;

use super::super::helpers::path;
use super::template::Template;

const DEFAULT_TEMPLATE: &'static str =
    "# List resources\n\nGET http://localhost/\nUser-Agent: Rustaman\n";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    BeingCreated,
    Active,
    Deleted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    id: usize,
    name: String,
    template: Template,
    status: Status,
}

impl Request {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }
    pub fn active(&self) -> bool {
        match self.status {
            Status::Active => true,
            _ => false,
        }
    }
    pub fn activate(&mut self) {
        self.status = Status::Active;
    }

    pub fn template(&self) -> &str {
        self.template.as_str()
    }
    pub fn set_template(&mut self, template: &str) {
        self.template = template.to_owned();
    }
}

pub type Requests = Vec<Request>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    name: String,
    requests: Requests,
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
                requests: vec![],
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
        let filepath =
            path::workspace("rustaman.json").expect("Cannot build default workspace filename");
        match Workspace::from_file(filepath.to_str().unwrap()) {
            Ok(res) => res,
            Err(err) => {
                error!(
                    "Error while loading workspace from filem creating default: {}",
                    err
                );
                let workspace = Workspace::new(filepath.to_str().unwrap());
                workspace.sync().unwrap();
                info!("New workspace created");
                workspace
            }
        }
    }

    pub fn sync(&self) -> Result<(), path::IOError> {
        info!("Writing workspace in file {}", self.filepath());
        let filecontent = serde_json::to_string_pretty(&self.payload);
        let filecontent =
            filecontent.expect("Unable to save workspace, cannot serilizing it to json");
        path::write_file(self.filepath(), filecontent.as_str())?;
        Ok(())
    }

    pub fn safe_sync(&self) {
        self.sync().unwrap_or_else(|err| {
            error!{"Workspace not synchronized: {}", err}
        });
    }

    fn filepath(&self) -> &str {
        self.filepath.as_str()
    }

    pub fn name(&self) -> &str {
        self.payload.name.as_str()
    }

    pub fn requests(&self) -> &[Request] {
        self.payload.requests.as_slice()
    }

    pub fn request(&self, id: usize) -> Option<&Request> {
        for request in self.requests().iter() {
            if request.id() == id {
                return Some(&request);
            }
        }
        None
    }

    pub fn create_request(&mut self) -> &Request {
        let id = match self.payload.requests.last() {
            None => 1,
            Some(req) => req.id + 1,
        };
        let name = format!("Req #{}", id);
        let request = Request {
            id: id,
            name: name.to_owned(),
            status: Status::BeingCreated,
            template: DEFAULT_TEMPLATE.to_owned(),
        };
        self.payload.requests.push(request);
        self.payload.requests.last().unwrap()
    }

    pub fn set_request_name(&mut self, id: usize, name: &str) {
        for request in self.payload.requests.iter_mut() {
            if request.id() == id {
                request.activate();
                request.set_name(name);
                break;
            }
        }
        self.safe_sync();
    }

    pub fn set_request_template(&mut self, id: usize, template: &str) {
        for request in self.payload.requests.iter_mut() {
            if request.id() == id {
                request.activate();
                request.set_template(template);
                break;
            }
        }
        self.safe_sync();
    }
}
