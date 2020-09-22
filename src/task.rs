use crate::{user::*, DB};
use database_table::DatabaseTable;
use rql::*;
use updatable::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Builder, Updatable)]
pub struct Task {
    title: String,
    description: String,

    assignees: Vec<Id<User>>,
    subtasks: Vec<Id<Task>>,
}
impl Task {
    pub fn new<S: ToString>(title: S) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            assignees: Vec::new(),
            subtasks: Vec::new(),
        }
    }
    pub fn with_subtasks<S: ToString>(title: S, subtasks: Vec<Id<Self>>) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            assignees: Vec::new(),
            subtasks,
        }
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn set_description<S: ToString>(&mut self, new_desc: S) {
        self.description = new_desc.to_string();
    }
    pub fn title(&self) -> &String {
        &self.title
    }
    pub fn set_title<S: ToString>(&mut self, new_title: S) {
        self.title = new_title.to_string();
    }
    pub fn assignees(&self) -> &Vec<Id<User>> {
        &self.assignees
    }
    pub fn add_assignee(&mut self, id: Id<User>) {
        self.assignees.push(id);
    }
    pub fn subtasks(&self) -> &Vec<Id<Self>> {
        &self.subtasks
    }
    pub fn children_mut(&mut self) -> &mut Vec<Id<Self>> {
        &mut self.subtasks
    }
}

impl<'a> DatabaseTable<'a> for Task {
    fn table() -> TableGuard<'a, Self> {
        DB.task()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.task_mut()
    }
}
