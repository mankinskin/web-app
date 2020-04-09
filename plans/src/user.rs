use serde_json::{
    *,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: usize,
    name: String,
}

impl User {
    pub fn new<S: ToString>(name: S) -> Self {
        Self {
            name: name.to_string(),
            id: 0,
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}
