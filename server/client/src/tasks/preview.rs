use crate::{
    page,
    route,
    tasks::*,
    config::*,
    root,
};
use database::{
    Entry,
};

impl Component for Model {
    type Msg = Msg;
}
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
impl Config<Model> for Entry<Task> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: task::Model::from(self),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
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
                msg.clone(),
                &mut model.task,
                &mut orders.proxy(Msg::Task)
            );
            //match msg {
            //    task::Msg::Edit => {
            //        page::go_to(route::Route::Task(model.task.task_id.clone()), orders);
            //    },
            //    _ => {}
            //}
        },
        Msg::Open => {
            page::go_to(route::Route::Task(model.task.task_id.clone()), orders);
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
                //button![
                //    simple_ev(Ev::Click, Msg::Task(task::Msg::Edit)),
                //    "Edit"
                //],
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
