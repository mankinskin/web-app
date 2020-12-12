use crate::user::*;
use database_table::{
    Entry,
    TableRoutable,
};
use enum_paths::AsPath;
use rql::*;
use serde::{
    Deserialize,
    Serialize,
};
#[cfg(target_arch = "wasm32")]
use {
    super::*,
    components::{
        entry,
        preview,
        Component,
        Edit,
        Viewable,
    },
    seed::{
        prelude::*,
        *,
    },
};

use derive_builder::Builder;
#[cfg(target_arch = "wasm32")]
pub mod editor;
#[cfg(target_arch = "wasm32")]
pub mod list;
#[cfg(target_arch = "wasm32")]
pub mod profile;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Builder)]
pub struct Task {
    title: String,
    description: String,

    assignees: Vec<Id<User>>,
    subtasks: Vec<Id<Task>>,
}
#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Tasks,
    #[as_path = ""]
    Task(Id<Task>),
}
impl database_table::Route for Route {}
impl TableRoutable for Task {
    type Route = Route;
    fn table_route() -> Route {
        Route::Tasks
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::Task(id)
    }
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
impl From<Entry<Task>> for Task {
    fn from(entry: Entry<Self>) -> Self {
        entry.into_inner()
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub enum Msg {
    SetDescription(String),
    SetTitle(String),
    Entry(Box<entry::Msg<Task>>),
}
#[cfg(target_arch = "wasm32")]
impl Component for Task {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::SetTitle(n) => {
                self.set_title(n);
            }
            Msg::SetDescription(d) => {
                self.set_description(d);
            }
            Msg::Entry(_) => {}
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Viewable for Task {
    fn view(&self) -> Node<Self::Msg> {
        div![p![self.title()],]
    }
}
#[cfg(target_arch = "wasm32")]
impl preview::Previewable for Task {
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
                self.title(),
            ],
            div![],
            p![
                style! {
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.subtasks().len(),
            p![
                style! {
                    St::Margin => "0",
                },
                "Assignees:"
            ],
            self.assignees().len(),
            button![
                ev(Ev::Click, |_| Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
#[cfg(target_arch = "wasm32")]
impl Edit for Task {
    fn edit(&self) -> Node<Msg> {
        div![
            label!["Title"],
            input![
                attrs! {
                    At::Placeholder => "Title",
                    At::Value => self.title(),
                },
                input_ev(Ev::Input, Msg::SetTitle)
            ],
            label!["Description"],
            textarea![
                attrs! {
                    At::Placeholder => "Description...",
                    At::Value => self.description(),
                },
                input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
