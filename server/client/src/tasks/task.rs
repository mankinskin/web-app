use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use rql::{
    Id,
};
use crate::{
    root::{
        GMsg,
    },
    tasks::*,
};
use database::{
    Entry,
};
use std::result::Result;

#[derive(Clone)]
pub enum Config {
    TaskId(Id<Task>),
    Model(Model),
    Entry(Entry<Task>),
}
impl From<Id<Task>> for Config {
    fn from(id: Id<Task>) -> Self {
        Self::TaskId(id)
    }
}
impl From<Model> for Config {
    fn from(model: Model) -> Self {
        Self::Model(model)
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::TaskId(id) => {
            orders.send_msg(Msg::Get(id.clone()));
            Model::from(id)
        },
        Config::Entry(entry) => {
            Model::from(entry)
        },
        Config::Model(model) => {
            model
        },
    }
}
#[derive(Clone)]
pub struct Model {
    pub task_id: Id<Task>,
    pub task: Option<Task>,
}
impl From<Id<Task>> for Model {
    fn from(id: Id<Task>) -> Self {
        Self {
            task_id: id,
            task: None,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task_id: *entry.id(),
            task: Some(entry.data().clone()),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Task>),
    Task(Result<Option<Task>, String>),

    Delete,
    Deleted(Result<Option<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Task(res) => {
            match res {
                Ok(t) => model.task = t,
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_task(id)
                    .map(|res| Msg::Task(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Delete => {
            orders.perform_cmd(
                api::delete_task(model.task_id)
                .map(|res| Msg::Deleted(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Deleted(_res) => {
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.task {
        Some(model) => {
            div![
                h1!["Task"],
                p![model.title()],
                button![
                    simple_ev(Ev::Click, Msg::Delete),
                    "Delete"
                ],
            ]
        },
        None => {
            div![
                h1!["Task"],
                p!["Loading..."],
            ]
        },
    }
}
