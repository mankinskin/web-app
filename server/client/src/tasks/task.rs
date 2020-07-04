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
    config::*,
    root::{
        GMsg,
    },
    tasks::*,
};
use database::{
    Entry,
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone)]
pub struct Model {
    pub task_id: Id<Task>,
    pub task: Option<Task>,
}
impl From<Id<Task>> for Msg {
    fn from(id: Id<Task>) -> Self {
        Msg::Get(id)
    }
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
    Task(Result<Option<Entry<Task>>, String>),

    Delete,
    Deleted(Result<Option<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Task(res) => {
            match res {
                Ok(r) =>
                    if let Some(entry) = r {
                        model.task_id = entry.id().clone();
                        model.task = Some(entry.data().clone());
                    },
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
    if let Some(model) = &model.task {
        div![
            h1!["Task"],
            p![model.title()],
            button![
                simple_ev(Ev::Click, Msg::Delete),
                "Delete"
            ],
            //button![
            //    simple_ev(Ev::Click, Msg::Edit),
            //    "Edit"
            //],
        ]
    } else {
        div![
            h1!["Task"],
            p!["Loading..."],
        ]
    }
}
