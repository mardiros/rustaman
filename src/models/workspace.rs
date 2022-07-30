use std::vec::Vec;



use super::super::errors::RustamanResult;
use super::super::helpers::path;
use super::environment::{Environment, Environments};
use super::status::Status;
use super::template::Template;

const DEFAULT_TEMPLATE: &str = "# TODO: DOCUMENT ME\n\nGET http://localhost/\n";

const DEFAULT_ENVIRONMENT: &str = "%YAML 1.2\n---\n";

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

    pub fn soft_delete(&mut self) {
        self.status = Status::Deleted;
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
    environments: Environments,
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
                environments: vec![Environment::new(1, "Dev", DEFAULT_ENVIRONMENT)],
            },
        }
    }

    pub fn from_file(filepath: &str) -> RustamanResult<Self> {
        info!("Try loading workspace from file {}", filepath);
        let file = std::fs::File::open(filepath)?;
        let reader = std::io::BufReader::new(file);
        let payload = serde_json::from_reader(reader).expect("Format error");
        debug!("Payload constructed from File {} readed by serde_json with BufReader", filepath);

        let workspace = Workspace {
            payload,
            filepath: filepath.to_string(),
        };
        info!("Workspace loaded from file {}", filepath);
        Ok(workspace)
    }

    pub fn default() -> Self {
        let filepath =
            path::workspace("workspace.json").expect("Cannot build default workspace filename");
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

    pub fn sync(&self) -> RustamanResult<()> {
        info!("Writing workspace in file {}", self.filepath());
        let filecontent = serde_json::to_string_pretty(&self.payload);
        let filecontent =
            filecontent.expect("Unable to save workspace, cannot serializing it to json");
        path::write_file(self.filepath(), filecontent.as_str())?;
        Ok(())
    }

    pub fn safe_sync(&self) {
        self.sync().unwrap_or_else(|err| {
            error! {"Workspace not synchronized: {}", err}
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
                return Some(request);
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
            id,
            name,
            status: Status::BeingCreated,
            template: DEFAULT_TEMPLATE.to_owned(),
        };
        self.payload.requests.push(request);
        self.payload.requests.last().unwrap()
    }

    pub fn set_request_name(&mut self, id: usize, name: &str) {
        for request in &mut self.payload.requests {
            if request.id() == id {
                request.activate();
                request.set_name(name);
                break;
            }
        }
        self.safe_sync();
    }

    pub fn delete_request(&mut self, id: usize) {
        for request in &mut self.payload.requests {
            if request.id() == id {
                request.soft_delete();
                break;
            }
        }
        self.safe_sync();
    }

    pub fn set_request_template(&mut self, id: usize, template: &str) {
        for request in &mut self.payload.requests {
            if request.id() == id {
                request.activate();
                request.set_template(template);
                break;
            }
        }
        self.safe_sync();
    }

    pub fn environments(&self) -> &[Environment] {
        self.payload.environments.as_slice()
    }

    pub fn environment(&self, id: usize) -> Option<&Environment> {
        for environment in self.environments().iter() {
            if environment.id() == id {
                return Some(environment);
            }
        }
        None
    }

    pub fn create_environment(&mut self, name: &str) -> &Environment {
        let id = match self.payload.environments.last() {
            None => 1,
            Some(env) => env.id() + 1,
        };
        let env = Environment::new(id, name, DEFAULT_ENVIRONMENT);
        self.payload.environments.push(env);
        let env = self.payload.environments.last().unwrap();
        self.safe_sync();
        env
    }

    pub fn set_environ_payload(&mut self, id: usize, payload: &str) {
        for environment in &mut self.payload.environments {
            if environment.id() == id {
                environment.set_payload(payload);
                break;
            }
        }
        self.safe_sync();
    }

    pub fn delete_environment(&mut self, id: usize) {
        for environment in &mut self.payload.environments {
            if environment.id() == id {
                environment.soft_delete();
                break;
            }
        }
        self.safe_sync();
    }
}
