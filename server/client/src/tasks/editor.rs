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
    tasks::{self, *},
};
use std::result::Result;

#[derive(Clone)]
pub struct Model {
    pub task: Task,
    pub task_id: Option<Id<Task>>,
    pub config: Config,
}
impl Model {
    fn empty() -> Self {
        Self {
            task: Task::new(String::new()),
            task_id: None,
            config: Config::Empty,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Config {
    Empty,
    ProjectId(Id<Project>),
}
impl From<Config> for Model {
    fn from(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}
impl Config {
    fn update(&self, _orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            _ => {}
        }
    }
}
impl From<tasks::Config> for Config {
    fn from(config: tasks::Config) -> Self {
        match config {
            tasks::Config::ProjectId(id) => {
                Self::ProjectId(id)
            },
            _ => Self::Empty,
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    config.update(orders);
    Model::from(config)
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Create,
    Cancel,
    CreatedTask(Result<Id<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::Create => {
            let task = model.task.clone();
            if let Config::ProjectId(id) = model.config {
                orders.perform_cmd(
                    api::project_create_subtask(id, task)
                        .map(|res| Msg::CreatedTask(res.map_err(|e| format!("{:?}", e))))
                );
            } else {
                orders.perform_cmd(
                    api::post_task(task)
                        .map(|res| Msg::CreatedTask(res.map_err(|e| format!("{:?}", e))))
                );
            }
        },
        Msg::CreatedTask(res) => {
            match res {
                Ok(id) => model.task_id = Some(id),
                Err(e) => { seed::log(e); },
            }
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
