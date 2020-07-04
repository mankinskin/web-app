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
        self,
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
impl Config<Model> for Id<Task> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task_id: self,
            task: None,
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::Get(self));
    }
}
impl Config<Model> for Entry<Task> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task_id: *self.id(),
            task: Some(self.data().clone()),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Task>),
    GotTask(Result<Option<Entry<Task>>, String>),

    Delete,
    Deleted(Result<Option<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_task(id)
                    .map(|res| Msg::GotTask(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::GotTask(res) => {
            match res {
                Ok(r) =>
                    if let Some(entry) = r {
                        model.task_id = entry.id().clone();
                        model.task = Some(entry.data().clone());
                    },
                Err(e) => { seed::log(e); },
            }
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
