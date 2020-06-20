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
    fetched::{
        self,
        Fetched,
        Query,
    },
    projects::*,
    fetched::{
        Status,
    },
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub project: fetched::Fetched<Project>,
}
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    fn ready(id: Id<Project>, project: Project) -> Self {
        Self {
            project: Fetched::ready(
                      url::Url::parse("http://localhost:8000/api/projects").unwrap(),
                      project,
                      Query::Id(id)
                      ),
        }
    }
    fn empty(query: Query<Project>) -> Self {
        Self {
            project:
                Fetched::empty(
                    url::Url::parse("http://localhost:8000/api/projects").unwrap(),
                    query,
                ),
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
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Query<Project>> for Model {
    fn from(query: Query<Project>) -> Self {
        Self::empty(query)
    }
}
impl From<Id<Project>> for Model {
    fn from(id: Id<Project>) -> Self {
        Self::from(Query::Id(id))
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetched::Msg<Project>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            model.project.update(msg, &mut orders.proxy(Msg::Fetch))
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model.project.status() {
        Status::Ready(model) => {
            div![
                h1!["Project"],
                p![model.name()],
            ]
        },
        Status::Waiting => {
            div![
                h1!["Project"],
                p!["Loading..."],
            ]
        },
        Status::Empty => {
            div![
                h1!["Empty Project"],
            ]
        },
        Status::Failed(s) => {
            div![
                format!("Failed: {}", s)
            ]
        },
    }
}
