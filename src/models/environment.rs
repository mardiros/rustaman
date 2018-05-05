use std::vec::Vec;

use super::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    id: usize,
    name: String,
    payload: String,
    status: Status,
}

impl Environment {
    pub fn new(id: usize, name: &str, payload: &str) -> Self {
        Environment {
            id,
            name: name.to_owned(),
            payload: payload.to_owned(),
            status: Status::Active,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn payload(&self) -> &str {
        self.payload.as_str()
    }
    pub fn set_payload(&mut self, payload: &str) {
        self.payload = payload.to_owned()
    }

    pub fn active(&self) -> bool {
        match self.status {
            Status::Active => true,
            _ => false,
        }
    }

    pub fn soft_delete(&mut self) {
        self.status = Status::Deleted;
    }
}

pub type Environments = Vec<Environment>;
