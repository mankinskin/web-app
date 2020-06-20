use crate::{
    page,
    projects::*,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub project: project::Model,
}
impl From<project::Model> for Model {
    fn from(model: project::Model) -> Self {
        Self {
            project: model,
        }
    }
}
impl From<&Entry<Project>> for Model {
    fn from(entry: &Entry<Project>) -> Self {
        Self {
            project: project::Model::from(entry),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Project(project::Msg),
    GoToProject,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Project(msg) => {
            project::update(
                msg,
                &mut model.project,
                &mut orders.proxy(Msg::Project)
            )
        },
        Msg::GoToProject => {
            seed::log!("GoToProject");
            page::go_to(project::Model::from(model.project.clone()), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model.project.project.status() {
        Status::Ready(project) => {
            div![
                a![
                    attrs!{
                        At::Href => "";
                    },
                    project.name(),
                    simple_ev(Ev::Click, Msg::GoToProject),
                ],
                p!["Preview"],
            ]
        },
        Status::Waiting => {
            div![
                h1!["Preview"],
                p!["Loading..."],
            ]
        },
        Status::Empty => {
            div![
                h1!["Empty Preview"],
            ]
        },
        Status::Failed(s) => {
            div![
                format!("Failed: {}", s)
            ]
        },
    }
}
