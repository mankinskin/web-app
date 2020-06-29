use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
    project::*,
};
use crate::{
    root::{
        GMsg,
    },
};
use database::{
    Entry,
};
use rql::{
    *,
};
use std::result::Result;

pub mod preview;
pub mod task;
pub mod editor;

#[derive(Clone)]
pub enum Config {
    Empty,
    All,
    ProjectId(Id<Project>),
}
impl Config {
    fn update(&self, orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            Config::All => {
                orders.send_msg(Msg::GetAll);
            },
            Config::ProjectId(id) => {
                orders.send_msg(Msg::GetProjectTasks(id.clone()));
            },
            _ => {},
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    config.update(orders);
    Model::from(config)
}
#[derive(Clone)]
pub struct Model {
    previews: Vec<preview::Model>,
    editor: Option<editor::Model>,
    config: Config,
}
impl Model {
    pub fn empty() -> Self {
        Self {
            previews: vec![],
            editor: None,
            config: Config::Empty,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
impl From<Config> for Model {
    fn from(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    GetAll,
    AllTasks(Result<Vec<Entry<Task>>, String>),
    Preview(usize, preview::Msg),
    OpenEditor,
    Editor(editor::Msg),
    GetProjectTasks(Id<Project>),
    ProjectTasks(Result<Vec<Entry<Task>>, String>),
}
fn init_previews(entries: Vec<Entry<Task>>, orders: &mut impl Orders<Msg, GMsg>) -> Vec<preview::Model> {
    entries
        .iter()
        .enumerate()
        .map(|(i, u)|
            preview::Model::from(
                task::init(
                    task::Config::Entry(u.clone()),
                    &mut orders
                        .proxy(move |p| Msg::Preview(i, preview::Msg::Task(p)))
                )
            )
        )
        .collect()
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
                model.config.update(orders);
            }
        },
        Msg::OpenEditor => {
            model.editor = Some(editor::init(editor::Config::from(model.config.clone()), &mut orders.proxy(Msg::Editor)));
        },
        Msg::Editor(msg) => {
            if let Some(ed) = &mut model.editor {
                editor::update(
                    msg.clone(),
                    ed,
                    &mut orders.proxy(Msg::Editor)
                );
                match msg {
                    editor::Msg::Cancel => {
                        model.editor = None;
                    },
                    editor::Msg::Create => {
                        model.config.update(orders);
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
                    simple_ev(Ev::Click, Msg::OpenEditor),
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
