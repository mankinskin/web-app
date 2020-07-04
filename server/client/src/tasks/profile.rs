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
    task: task::Model,
    editor: Option<editor::Model>,
}
impl From<Id<Task>> for Msg {
    fn from(id: Id<Task>) -> Self {
        Msg::Get(id)
    }
}
impl From<Id<Task>> for Model {
    fn from(id: Id<Task>) -> Self {
        Self {
            task: task::Model::from(id),
            editor: None,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task: task::Model::from(entry),
            editor: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Task>),
    GotTask(Result<Option<Entry<Task>>, String>),

    Delete,
    Deleted(Result<Option<Task>, String>),

    Edit,
    Editor(editor::Msg),

    Task(task::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::GotTask(res) => {
            match res {
                Ok(r) =>
                    if let Some(entry) = r {
                        model.task.task_id = entry.id().clone();
                        model.task.task = Some(entry.data().clone());
                    },
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_task(id)
                    .map(|res| Msg::GotTask(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Delete => {
            orders.perform_cmd(
                api::delete_task(model.task.task_id)
                .map(|res| Msg::Deleted(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Deleted(_res) => {
        },
        Msg::Edit => {
            model.editor = Some(Config::init(model.task.clone(), &mut orders.proxy(Msg::Editor)));
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
        Msg::Task(msg) => {
            task::update(
                msg,
                &mut model.task,
                &mut orders.proxy(Msg::Task),
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(model) = &model.editor {
        div![
            editor::view(&model)
                .map_msg(Msg::Editor)
        ]
    } else {
        task::view(&model.task)
            .map_msg(Msg::Task)
    }
}
