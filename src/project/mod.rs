use super::*;
use crate::{
    task::*,
    user::*,
    DB,
};
#[cfg(target_arch = "wasm32")]
use components::{
    entry,
    preview,
    Component,
    Edit,
    Viewable,
};
use database_table::{
    DatabaseTable,
    RemoteTable,
    TableRoutable,
    Entry,
};
use rql::*;
#[cfg(target_arch = "wasm32")]
use seed::{
    browser::fetch::{
        fetch,
        Request,
        Method,
    },
    prelude::*,
    *,
};
use serde::{
    Deserialize,
    Serialize,
};
use derive_builder::Builder;

#[cfg(target_arch = "wasm32")]
pub mod editor;
#[cfg(target_arch = "wasm32")]
pub mod list;
#[cfg(target_arch = "wasm32")]
pub mod profile;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Builder)]
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

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl RemoteTable for Project {
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        fetch(
            Request::new(Self::entry_route(id))
                .method(Method::Get)
        ).await?
        .json().await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        fetch(
            Request::new(Self::entry_route(id))
                .method(Method::Delete)
        ).await?
        .json().await
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        fetch(
            Request::new(Self::table_route())
                .method(Method::Get)
        ).await?
        .json().await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        fetch(
            Request::new(Self::table_route())
                .method(Method::Post)
                .json(data).await?
        ).await?
        .json().await
    }
}
impl From<Entry<Project>> for Project {
    fn from(entry: Entry<Self>) -> Self {
        entry.into_inner()
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
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
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
            }
            Msg::SetDescription(d) => {
                self.set_description(d);
            }
            Msg::Entry(_) => {}
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Viewable for Project {
    fn view(&self) -> Node<Self::Msg> {
        div![p![self.name()],]
    }
}
#[cfg(target_arch = "wasm32")]
impl preview::Preview for Project {
    fn preview(&self) -> Node<Msg> {
        div![
            style! {
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
                St::Cursor => "pointer",
            },
            h3![
                style! {
                    St::Margin => "0",
                },
                self.name(),
            ],
            div![],
            p![
                style! {
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.tasks().len(),
            p![
                style! {
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
            label!["Name"],
            input![
                attrs! {
                    At::Placeholder => "Name",
                    At::Value => self.name(),
                },
                input_ev(Ev::Input, Msg::SetName)
            ],
            label!["Description"],
            textarea![
                attrs! {
                    At::Placeholder => "Description",
                    At::Value => self.description(),
                },
                input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
