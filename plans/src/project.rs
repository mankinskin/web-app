use crate::{
    user::{
        *,
    },
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

#[derive(Clone,
         Debug,
         Serialize,
         Deserialize,
         Default,
         Builder,
         Updatable,
         PartialEq,
         )]
pub struct Project {
    name: String,
    members: Vec<Id<User>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            members: vec![],
        }
    }
    pub fn members(&self) -> &Vec<Id<User>> {
        &self.members
    }
    pub fn add_member(&mut self, id: Id<User>) {
        self.members.push(id);
    }
}
