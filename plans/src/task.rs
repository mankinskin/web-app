use crate::{
    user::*,
};
#[derive(Clone, Debug, Serialize, Deserialize, Builder, Default)]
pub struct Task {
    title: String,
    description: String,
    assignees: Vec<User>,
    children: Vec<Task>,
}

impl Task {
    pub fn new<S: ToString>(title: S) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            assignees: Vec::new(),
            children: Vec::new(),
        }
    }
    pub fn with_children<S: ToString>(title: S, children: Vec<Self>) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            assignees: Vec::new(),
            children,
        }
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn update_description<S: ToString>(&mut self, new_desc: S) {
        self.description = new_desc.to_string();
    }
    pub fn title(&self) -> &String {
        &self.title
    }
    pub fn update_title<S: ToString>(&mut self, new_title: S) {
        self.title = new_title.to_string();
    }
    pub fn assignees(&self) -> &Vec<User> {
        &self.assignees
    }
    pub fn add_assignee(&self, ) -> &Vec<User> {
        &self.assignees
    }
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }
}
