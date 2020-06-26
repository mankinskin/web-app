use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use crate::{
    root::{
        GMsg,
    },
    tasks::*,
};
use rql::{
    *,
};
use std::result::Result;

#[derive(Clone)]
pub struct Model {
    pub task: Task,
    pub task_id: Option<Id<Task>>,
}
impl Model {
    fn empty() -> Self {
        Self {
            task: Task::new(String::new()),
            task_id: None,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Create,
    Cancel,
    CreateTask(Task),
    CreatedTask(Result<Id<Task>, String>),
}
impl Msg {
    pub fn post_task(task: &Task) -> Msg {
        Msg::CreateTask(task.clone())
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::CreatedTask(res) => {
            match res {
                Ok(id) => model.task_id = Some(id),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::CreateTask(t) => {
            orders.perform_cmd(
                api::post_task(t)
                    .map(|res| Msg::CreatedTask(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Create => {
            orders.send_msg(Msg::post_task(&model.task));
        },
        Msg::Cancel => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    form![
        h1!["New Task"],
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
        // Cancel Button
        button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
        // Create Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Create"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Create
        }),
    ]
}
