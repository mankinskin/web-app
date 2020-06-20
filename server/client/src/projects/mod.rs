use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    root::{
        GMsg,
    },
    fetched::{
        self,
        Status,
        Fetched,
        Query,
    },
};
use database::{
    Entry,
};

pub mod preview;
pub mod project;

#[derive(Clone)]
pub struct Model {
    projects: Fetched<Vec<Entry<Project>>>,
    previews: Vec<preview::Model>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            projects:
                Fetched::empty(
                       url::Url::parse("http://localhost:8000/api/projects").unwrap(),
                       Query::all()
                ),
            previews: vec![],
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetched::Msg<Vec<Entry<Project>>>),
    Preview(usize, preview::Msg),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            model.projects.update(msg, &mut orders.proxy(Msg::Fetch));
            if let Status::Ready(projects) = model.projects.status() {
                model.previews = projects.iter().map(|u| preview::Model::from(u)).collect()
            }
        },
        Msg::Preview(index, msg) => {
            preview::update(
                msg,
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.projects.status() {
        Status::Ready(projects) => {
            div![
                ul![
                    projects.iter().enumerate()
                        .map(|(i, entry)| li![
                             preview::view(&preview::Model::from(entry))
                                .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                        ])
                ]
            ]
        },
        Status::Waiting => {
            div![
                format!("Fetching...")
            ]
        },
        Status::Empty => {
            div![
                format!("Empty...")
            ]
        },
        Status::Failed(s) => {
            div![
                format!("Failed: {}", s)
            ]
        },
    }
}
