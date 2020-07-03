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
    Task(task::Config),
    Editor(editor::Config),
}
impl From<task::Config> for Config {
    fn from(config: task::Config) -> Self {
        Self::Task(config)
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::Task(config) => {
            Model::Task(task::init(config, orders.proxy(Msg::Task)))
        },
        Config::Editor(config) => {
            Model::Editor(editor::init(config, orders.proxy(Msg::Editor)))
        },
    }
}
#[derive(Clone)]
pub enum Model {
    Task(task::Model),
    Editor(editor::Model),
}
impl From<Id<Task>> for Model {
    fn from(id: Id<Task>) -> Self {
        Self {
            task_id: id,
            task: None,
            editor: None,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task_id: *entry.id(),
            task: Some(entry.data().clone()),
            editor: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Task>),
    Task(Result<Option<Task>, String>),

    Delete,
    Deleted(Result<Option<Task>, String>),

    Edit,
    Editor(editor::Msg),
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
        Msg::Edit => {
            if let Some(task) = &model.task {
                model.editor = Some(editor::init(editor::Config::Task(task.clone()), &mut orders.proxy(Msg::Editor)));
            }
        },
        Msg::Editor(msg) => {
            if let Some(model) = &mut model.editor {
                editor::update(
                    msg,
                    model,
                    &mut orders.proxy(Msg::Editor),
                );
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(model) = &model.editor {
        div![
            editor::view(&model).map_msg(Msg::Editor)
        ]
    } else {
        if let Some(model) = &model.task {
            div![
                h1!["Task"],
                p![model.title()],
                button![
                    simple_ev(Ev::Click, Msg::Delete),
                    "Delete"
                ],
                button![
                    simple_ev(Ev::Click, Msg::Edit),
                    "Edit"
                ],
            ]
        } else {
            div![
                h1!["Task"],
                p!["Loading..."],
            ]
        }
    }
}
