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
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    fn ready(id: Id<Project>, project: Project) -> Self {
        Self {
            project_id: id,
            project: Some(project),
            tasks: tasks::Model::empty(),
        }
    }
    fn fetch(id: Id<Project>) -> Self {
        Self {
            project_id: id,
            project: None,
            tasks: tasks::Model::empty(),
        }
    }
}
impl From<&Entry<Project>> for Model {
    fn from(entry: &Entry<Project>) -> Self {
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Entry<Project>> for Model {
    fn from(entry: Entry<Project>) -> Self {
        Self::from(&entry)
    }
}
impl From<Id<Project>> for Model {
    fn from(id: Id<Project>) -> Self {
        Self::fetch(id)
    }
}
#[derive(Clone)]
pub enum Config {
    ProjectId(Id<Project>),
    Model(Model),
}
impl From<Id<Project>> for Config {
    fn from(id: Id<Project>) -> Self {
        Self::ProjectId(id)
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
            Model::fetch(id)
        },
        Config::Model(model) => {
            model
        },
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Project>),
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
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!["Project"],
        match &model.project {
            Some(model) => {
                div![
                    p![model.name()],
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
