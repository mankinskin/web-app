use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
    project::*,
};
use crate::{
    config::*,
    root::{
        self,
        GMsg,
    },
    tasks::{
        preview,
        task,
        editor,
    },
};
use database::{
    Entry,
};
use rql::{
    *,
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        match self {
            _ => Model::default(),
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(self);
    }
}
impl Config<Model> for Vec<Entry<Task>> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            previews: init_previews(self, orders),
            editor: None,
            project_id: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Id<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            project_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::GetProjectTasks(self));
    }
}
fn init_previews(entries: Vec<Entry<Task>>, orders: &mut impl Orders<Msg, GMsg>) -> Vec<preview::Model> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)|
            Config::init(
                entry.clone(),
                &mut orders
                    .proxy(move |msg| Msg::Preview(i, msg))
            )
        )
        .collect()
}
#[derive(Clone, Default)]
pub struct Model {
    previews: Vec<preview::Model>,
    editor: Option<editor::Model>,
    project_id: Option<Id<Project>>,
}
#[derive(Clone)]
pub enum Msg {
    GetAll,
    AllTasks(Result<Vec<Entry<Task>>, String>),
    Preview(usize, preview::Msg),
    Editor(editor::Msg),
    GetProjectTasks(Id<Project>),
    ProjectTasks(Result<Vec<Entry<Task>>, String>),
    NewTask,
    CreatedTask(Result<Id<Task>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::GetAll => {
            orders.perform_cmd(
                api::get_tasks()
                    .map(|res| Msg::AllTasks(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::AllTasks(res) => {
            match res {
                Ok(ps) => model.previews = init_previews(ps, orders),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::GetProjectTasks(id) => {
            orders.perform_cmd(
                api::get_project_tasks(id)
                    .map(|res| Msg::ProjectTasks(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::ProjectTasks(res) => {
            match res {
                Ok(ps) => model.previews = init_previews(ps, orders),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Preview(index, msg) => {
            preview::update(
                msg.clone(),
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
            if let preview::Msg::Task(task::Msg::Deleted(_)) = msg {
                model.previews.remove(index);
            }
        },
        Msg::NewTask => {
            model.editor = Some(editor::Model::default());
        },
        Msg::CreatedTask(res) => {
            match res {
                Ok(_id) => {},
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Editor(msg) => {
            if let Some(editor) = &mut model.editor {
                editor::update(
                    msg.clone(),
                    editor,
                    &mut orders.proxy(Msg::Editor)
                );
                match msg {
                    editor::Msg::Cancel => {
                        model.editor = None;
                    },
                    editor::Msg::Submit => {
                        let task = editor.task.clone();
                        if let Some(id) = model.project_id {
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
                        orders.send_msg(
                            if let Some(id) = model.project_id {
                                Msg::GetProjectTasks(id)
                            } else {
                                Msg::GetAll
                            }
                        );
                    },
                    _ => {},
                }
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        if let Some(model) = &model.editor {
            editor::view(&model).map_msg(Msg::Editor)
        } else {
            if let Some(_) = api::auth::get_session() {
                button![
                    simple_ev(Ev::Click, Msg::NewTask),
                    "New Task"
                ]
            } else { empty![] }
        },
        ul![
            model.previews.iter().enumerate()
                .map(|(i, preview)| li![
                     preview::view(&preview)
                        .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                ])
        ]
    ]
}
