use crate::{task::*, user::*, DB};
use database_table::DatabaseTable;
use rql::*;
use serde::{Deserialize, Serialize};
use updatable::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Builder, Updatable)]
pub struct Project {
    name: String,
    description: String,
    members: Vec<Id<User>>,
    tasks: Vec<Id<Task>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            members: vec![],
            tasks: vec![],
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn set_name(&mut self, n: String) {
        self.name = n;
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn set_description(&mut self, n: String) {
        self.description = n;
    }
    pub fn members(&self) -> &Vec<Id<User>> {
        &self.members
    }
    pub fn add_member(&mut self, id: Id<User>) {
        self.members.push(id);
    }
    pub fn tasks(&self) -> &Vec<Id<Task>> {
        &self.tasks
    }
    pub fn add_task(&mut self, id: Id<Task>) {
        self.tasks.push(id);
    }
}

impl<'a> DatabaseTable<'a> for Project {
    fn table() -> TableGuard<'a, Self> {
        DB.project()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.project_mut()
    }
}
