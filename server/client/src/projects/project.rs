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
    fetch::{
        self,
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
        Self {
            project_id: id,
            project: None,
            tasks: tasks::Model::empty(),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetch::Msg<Project>),
    Tasks(tasks::Msg),
}
impl From<fetch::Msg<Project>> for Msg {
    fn from(msg: fetch::Msg<Project>) -> Self {
        Msg::Fetch(msg)
    }
}
impl From<tasks::Msg> for Msg {
    fn from(msg: tasks::Msg) -> Self {
        Msg::Tasks(msg)
    }
}
impl Msg {
    pub fn fetch_project(id: Id<Project>) -> Msg {
        Msg::Fetch(fetch::Msg::Request(fetch::Request::Get(Query::Id(id))))
    }
    pub fn fetch_tasks() -> Msg {
        Msg::Tasks(tasks::Msg::fetch_all())
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/projects").unwrap(),
                            request
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        fetch::Response::Get(data) => {
                            model.project = Some(data);
                        },
                        _ => {}
                    }
                    orders.send_msg(Msg::fetch_tasks());
                },
                fetch::Msg::Error(error) => {
                    seed::log(error);
                },
            }
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
