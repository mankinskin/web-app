use super::*;
use database_table::{
    DatabaseTable,
    TableItem,
    TableRoutable,
};
use rql::*;
use serde::{Deserialize, Serialize};
use updatable::*;
#[cfg(target_arch = "wasm32")]
use seed::{
    *,
    prelude::*,
};
#[cfg(target_arch = "wasm32")]
use components::{
    Component,
    Viewable,
    Edit,
    entry,
    preview,
};
use crate::{
    task::*,
    user::*,
    DB,
};

#[cfg(target_arch = "wasm32")]
pub mod editor;
#[cfg(target_arch = "wasm32")]
pub mod list;
#[cfg(target_arch = "wasm32")]
pub mod profile;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Builder, Updatable)]
pub struct Project {
    name: String,
    description: String,
    members: Vec<Id<User>>,
    tasks: Vec<Id<Task>>,
}
impl TableRoutable for Project {
    type Route = Route;
    fn table_route() -> Route {
        Route::Projects
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::Project(id)
    }
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

impl TableItem for Project {}
impl<'a> DatabaseTable<'a> for Project {
    fn table() -> TableGuard<'a, Self> {
        DB.project()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.project_mut()
    }
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub enum Msg {
    SetName(String),
    SetDescription(String),
    Entry(Box<entry::Msg<Project>>),
}
#[cfg(target_arch = "wasm32")]
impl Component for Project {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::SetName(n) => {
                self.set_name(n);
            },
            Msg::SetDescription(d) => {
                self.set_description(d);
            },
            Msg::Entry(_) => {}
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Viewable for Project {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p![self.name()],
        ]
    }
}
#[cfg(target_arch = "wasm32")]
impl preview::Preview for Project {
    fn preview(&self) -> Node<Msg> {
        div![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
                St::Cursor => "pointer",
            },
            h3![
                style!{
                    St::Margin => "0",
                },
                self.name(),
            ],
            div![],
            p![
                style!{
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.tasks().len(),
            p![
                style!{
                    St::Margin => "0",
                },
                "Members:"
            ],
            self.members().len(),
            button![
                ev(Ev::Click, |_| Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
#[cfg(target_arch = "wasm32")]
impl Edit for Project {
    fn edit(&self) -> Node<Msg> {
        div![
            label![
                "Name"
            ],
            input![
                attrs!{
                    At::Placeholder => "Name",
                    At::Value => self.name(),
                },
                input_ev(Ev::Input, Msg::SetName)
            ],
            label![
                "Description"
            ],
            textarea![
                attrs!{
                    At::Placeholder => "Description",
                    At::Value => self.description(),
                },
                input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
