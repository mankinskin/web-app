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
    fetch::{
        self,
        Query,
        Request,
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
    previews: Vec<preview::Model>,
    project_editor: Option<editor::Model>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            previews: vec![],
            project_editor: None,
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    Fetch(fetch::Msg<Vec<Entry<Project>>>),
    Preview(usize, preview::Msg),
    OpenProjectEditor,
    Editor(editor::Msg),
}
impl Msg {
    pub fn fetch_projects() -> Msg {
        Msg::Fetch(fetch::Msg::Request(Request::Get(Query::All)))
    }
}

impl From<fetch::Msg<Vec<Entry<Project>>>> for Msg {
    fn from(msg: fetch::Msg<Vec<Entry<Project>>>) -> Self {
        Msg::Fetch(msg)
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
                            request,
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        fetch::Response::Get(data) => {
                            model.previews = data.iter().map(|u| preview::Model::from(u)).collect()
                        },
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {

                },
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
            match msg {
                editor::Msg::Cancel => {
                    model.project_editor = None;
                },
                _ => {},
            }
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
            model.previews.iter().enumerate()
                .map(|(i, preview)| li![
                     preview::view(&preview)
                        .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                ])
        ]
    ]
}
