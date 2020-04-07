#[derive(Clone, Debug)]
pub struct Task {
    pub title: String,
    pub description: String,
    pub assignees: Vec<String>,
}

impl Task {
    pub fn new<S: ToString>(title: S) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            assignees: Vec::new(),
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
}

#[derive(Clone, Debug)]
pub struct TaskTree {
    task: Task,
    children: Vec<TaskTree>,
}
impl TaskTree {
    pub fn new(task: Task, children: Vec<Self>) -> Self {
        Self {
            task,
            children,
        }
    }
    pub fn task(&self) -> &Task {
        &self.task
    }
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }
}
