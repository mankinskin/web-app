use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use crate::{
    config::*,
    root::{
        self,
        GMsg,
    },
    task::{*},
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone, Default)]
pub struct Model {
    pub task: Task,
    pub task_id: Option<Id<Task>>,
    pub project_id: Option<Id<Project>>,
}
impl Config<Model> for Id<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            project_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for task::Model {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: self.task.unwrap_or(Default::default()),
            task_id: Some(self.task_id),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task_id: Some(entry.id().clone()),
            task: entry.data().clone(),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Cancel,
    Submit,
    Created(Result<Id<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::Cancel => {},
        Msg::Submit => {
            let task = model.task.clone();
            if let Some(id) = model.project_id {
                orders.perform_cmd(
                    api::project_create_subtask(id, task)
                        .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
                );
            } else {
                orders.perform_cmd(
                    api::post_task(task)
                        .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
                );
            }
        },
        Msg::Created(res) => {
            match res {
                Ok(id) => model.task_id = Some(id),
                Err(e) => { seed::log(e); },
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    form![
        style!{
            St::Display => "grid",
            St::GridTemplateColumns => "1fr",
            St::GridGap => "10px",
            St::MaxWidth => "20%",
        },
        if let Some(_) = model.task_id {
            h1!["Edit Task"]
        } else {
            h1!["New Task"]
        },
        label![
            "Title"
        ],
        input![
            attrs!{
                At::Placeholder => "Title",
                At::Value => model.task.title(),
            },
            input_ev(Ev::Input, Msg::ChangeTitle)
        ],
        label![
            "Description"
        ],
        textarea![
            attrs!{
                At::Placeholder => "Description...",
                At::Value => model.task.description(),
            },
            input_ev(Ev::Input, Msg::ChangeDescription)
        ],
        // Submit Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Create"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Submit
        }),
        // Cancel Button
        button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
    ]
}
