use crate::{
    page,
    tasks::*,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub task: task::Model,
}
impl From<task::Model> for Model {
    fn from(model: task::Model) -> Self {
        Self {
            task: model,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task: task::Model::from(entry),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Task(task::Msg),
    Open,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Task(msg) => {
            task::update(
                msg,
                &mut model.task,
                &mut orders.proxy(Msg::Task)
            )
        },
        Msg::Open => {
            page::go_to(task::Config::from(model.task.clone()), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.task.task {
        Some(task) => {
            div![
                a![
                    attrs!{
                        At::Href => "";
                    },
                    task.title(),
                    simple_ev(Ev::Click, Msg::Open),
                ],
                p!["Preview"],
                button![
                    simple_ev(Ev::Click, Msg::Task(task::Msg::Delete)),
                    "Delete"
                ],
            ]
        },
        None => {
            div![
                h1!["Preview"],
                p!["Loading..."],
            ]
        },
    }
}
