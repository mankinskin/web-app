use crate::{
    user::*,
    task::*,
};
use rql::{
    *,
};
use updatable::{
    *,
};
use serde::{
    Serialize,
    Deserialize,
};

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    Builder,
    Updatable,
    )]
pub struct Project {
    name: String,
    members: Vec<Id<User>>,
    tasks: Vec<Id<Task>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            members: vec![],
            tasks: vec![],
        }
    }
    pub fn name(&self) -> &String {
        &self.name
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
