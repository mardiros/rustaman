#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    name: String,
    payload: String,
}

impl Environment {
    pub fn new(name: &str, payload: &str) -> Self {
        Environment {
            name: name.to_owned(),
            payload: payload.to_owned(),
        }
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
}
