use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    root::{
        self,
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
pub mod editor;

#[derive(Clone)]
pub struct Model {
    projects: Fetched<Vec<Entry<Project>>>,
    previews: Vec<preview::Model>,
    project_editor: Option<editor::Model>,
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
            project_editor: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetched::Msg<Vec<Entry<Project>>>),
    Preview(usize, preview::Msg),
    OpenProjectEditor,
    Editor(editor::Msg),
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
        Msg::OpenProjectEditor => {
            model.project_editor = Some(editor::Model::default());
        },
        Msg::Editor(msg) => {
            if let Some(model) = &mut model.project_editor {
                editor::update(
                    msg,
                    model,
                    &mut orders.proxy(Msg::Editor)
                );
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.projects.status() {
        Status::Ready(projects) => {
            div![
                if let Some(model) = &model.project_editor {
                    editor::view(&model).map_msg(Msg::Editor)
                } else {
                    if let Some(_) = root::get_session() {
                        button![
                            simple_ev(Ev::Click, Msg::OpenProjectEditor),
                            "New Project"
                        ]
                    } else { empty![] }
                },
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
