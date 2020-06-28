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
pub struct Model {
    pub task_id: Id<Task>,
    pub task: Option<Task>,
}
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    fn ready(id: Id<Task>, task: Task) -> Self {
        Self {
            task_id: id,
            task: Some(task),
        }
    }
}
impl From<&Entry<Task>> for Model {
    fn from(entry: &Entry<Task>) -> Self {
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self::from(&entry)
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
#[derive(Clone)]
pub enum Msg {
    Get(Id<Task>),
    Task(Result<Option<Task>, String>),
}
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
impl Msg {
    pub fn fetch_task(id: Id<Task>) -> Msg {
        Msg::Get(id)
    }
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
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.task {
        Some(model) => {
            div![
                h1!["Task"],
                p![model.title()],
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
