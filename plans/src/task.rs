
#[derive(Clone, Debug)]
pub struct Task {
    description: String,
}

impl Task {
    pub fn new<S: ToString>(description: S) -> Self {
        Self {
            description: description.to_string(),
        }
    }
    pub fn description(&self) -> &String {
        &self.description
    }
}
