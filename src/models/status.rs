#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    BeingCreated,
    Active,
    Deleted,
}
