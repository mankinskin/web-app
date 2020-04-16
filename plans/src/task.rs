#[derive(Clone, Debug)]
pub struct Task {
    pub title: String,
    pub description: String,
    pub assignees: Vec<String>,
    pub children: Vec<Task>,
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
    pub fn title(&self) -> &String {
        &self.title
    }
    pub fn assignees(&self) -> &Vec<String> {
        &self.assignees
    }
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }
}
