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
pub enum Config {
    Empty,
    Task(task::Config),
}
impl Default for Config {
    fn default() -> Self {
        Self::Empty
    }
}
impl From<task::Config> for Config {
    fn from(config: task::Config) -> Self {
        Self::Task(config)
    }
}
impl Config {
    fn update(&self, _orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            _ => {}
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    config.update(orders);
    Model::from(config)
}
#[derive(Clone)]
pub struct Model {
    task: task::Model,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            task: task::Model::default(),
        }
    }
}
impl From<Task> for Model {
    fn from(task: Task) -> Self {
        Self {
            task,
            task_id: None,
        }
    }
}
impl From<Config> for Model {
    fn from(config: Config) -> Self {
        match config {
            Config::Empty => {
                Self::default()
            },
            Config::Task(task) => {
                Self::from(task)
            },
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Submit,
    Cancel,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::Submit => {
        },
        Msg::Cancel => {},
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
