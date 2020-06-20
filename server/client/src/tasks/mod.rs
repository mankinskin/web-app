use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
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
pub mod task;
pub mod editor;

#[derive(Clone)]
pub struct Model {
    previews: Vec<preview::Model>,
    editor: Option<editor::Model>,
}
impl Model {
    pub fn empty() -> Self {
        Self {
            previews: vec![],
            editor: None,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone)]
pub enum Msg {
    Fetch(fetch::Msg<Vec<Entry<Task>>>),
    Preview(usize, preview::Msg),
    OpenEditor,
    Editor(editor::Msg),
}
impl Msg {
    pub fn fetch_all() -> Msg {
        Msg::Fetch(fetch::Msg::Request(Request::Get(Query::All)))
    }
}

impl From<fetch::Msg<Vec<Entry<Task>>>> for Msg {
    fn from(msg: fetch::Msg<Vec<Entry<Task>>>) -> Self {
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
                            url::Url::parse("http://localhost:8000/api/tasks").unwrap(),
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
                    seed::log(error);
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
        Msg::OpenEditor => {
            model.editor = Some(editor::Model::default());
        },
        Msg::Editor(msg) => {
            if let Some(model) = &mut model.editor {
                editor::update(
                    msg.clone(),
                    model,
                    &mut orders.proxy(Msg::Editor)
                );
            }
            match msg {
                editor::Msg::Cancel => {
                    model.editor = None;
                },
                editor::Msg::Create => {
                    orders.send_msg(Msg::fetch_all());
                },
                _ => {},
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        if let Some(model) = &model.editor {
            editor::view(&model).map_msg(Msg::Editor)
        } else {
            if let Some(_) = root::get_session() {
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
