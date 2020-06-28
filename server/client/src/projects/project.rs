use seed::{
    prelude::*,
};
use plans::{
    project::*,
};
use rql::{
    Id,
};
use crate::{
    root::{
        GMsg,
    },
    projects::*,
    tasks,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub project_id: Id<Project>,
    pub project: Option<Project>,
    pub tasks: tasks::Model,
}
#[derive(Clone)]
pub enum Config {
    ProjectId(Id<Project>),
    Entry(Entry<Project>),
    Model(Model),
}
impl From<Id<Project>> for Config {
    fn from(id: Id<Project>) -> Self {
        Self::ProjectId(id)
    }
}
impl From<Entry<Project>> for Config {
    fn from(entry: Entry<Project>) -> Self {
        Self::Entry(entry)
    }
}
impl From<Model> for Config {
    fn from(model: Model) -> Self {
        Self::Model(model)
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::ProjectId(id) => {
            orders.send_msg(Msg::Get(id));
            Model {
                project_id: id,
                project: None,
                tasks: tasks::init(tasks::Config::Project(id), &mut orders.proxy(Msg::Tasks)),
            }
        },
        Config::Model(model) => {
            model
        },
        Config::Entry(entry) => {
            let id = entry.id().clone();
            let data = entry.data().clone();
            Model {
                project_id: id,
                project: Some(data),
                tasks: tasks::init(tasks::Config::Project(id), &mut orders.proxy(Msg::Tasks)),
            }
        },
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Project>),
    Delete,
    Deleted(Result<Option<Project>, String>),
    Project(Result<Option<Project>, String>),
    Tasks(tasks::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Project(res) => {
            match res {
                Ok(p) => model.project = p,
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_project(id)
                    .map(|res| Msg::Project(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Tasks(msg) => {
            tasks::update(
                msg,
                &mut model.tasks,
                &mut orders.proxy(Msg::Tasks)
            );
        },
        Msg::Deleted(res) => {
        },
        Msg::Delete => {
            orders.perform_cmd(
                api::delete_project(model.project_id)
                .map(|res| Msg::Deleted(res.map_err(|e| format!("{:?}", e))))
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!["Project"],
        match &model.project {
            Some(model) => {
                div![
                    p![model.name()],
                    button![
                        simple_ev(Ev::Click, Msg::Delete),
                        "Delete"
                    ],
                ]
            },
            None => {
                div![
                    p!["Loading..."],
                ]
            },
        },
        tasks::view(&model.tasks).map_msg(Msg::Tasks)
    ]
}
