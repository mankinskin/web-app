use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
    user::*,
};
use rql::{
    *,
};
use crate::{
    root::{
        GMsg,
    },
};
use database::{
    Entry,
};
use std::result::Result;

pub mod preview;
pub mod project;
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
    GetAll,
    AllProjects(Result<Vec<Entry<Project>>, String>),
    FetchUserProjects(Id<User>),
    UserProjects(Result<Vec<Entry<Project>>, String>),
    Preview(usize, preview::Msg),
    OpenEditor,
    Editor(editor::Msg),
}
impl Msg {
    pub fn fetch_all() -> Msg {
        Msg::GetAll
    }
    pub fn fetch_for_user(id: Id<User>) -> Msg {
        Msg::FetchUserProjects(id)
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::AllProjects(res) => {
            match res {
                Ok(ps) => model.previews = ps.iter().map(|u| preview::Model::from(u)).collect(),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::GetAll => {
            orders.perform_cmd(
                api::get_projects()
                    .map(|res| Msg::AllProjects(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::UserProjects(res) => {
            match res {
                Ok(ps) => model.previews = ps.iter().map(|u| preview::Model::from(u)).collect(),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::FetchUserProjects(id) => {
            orders.perform_cmd(
                api::find_user_projects(id)
                .map(|res| Msg::UserProjects(res.map_err(|e| format!("{:?}", e))))
            );
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
            if let Some(_) = api::auth::get_session() {
                button![
                    simple_ev(Ev::Click, Msg::OpenEditor),
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
