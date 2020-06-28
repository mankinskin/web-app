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
    config: Config,
}
impl Model {
    pub fn empty() -> Self {
        Self {
            previews: vec![],
            editor: None,
            config: Config::Empty,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
impl From<Config> for Model {
    fn from(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Config {
    Empty,
    All,
    User(Id<User>),
}
impl Config {
    fn update(&self, orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            Config::User(id) => {
                orders.send_msg(Msg::FetchUserProjects(id.clone()));
            },
            Config::All => {
                orders.send_msg(Msg::GetAll);
            },
            _ => {}
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    config.update(orders);
    Model::from(config)
}
#[derive(Clone)]
pub enum Msg {
    GetAll,
    OpenEditor,
    Editor(editor::Msg),
    AllProjects(Result<Vec<Entry<Project>>, String>),
    FetchUserProjects(Id<User>),
    UserProjects(Result<Vec<Entry<Project>>, String>),
    Preview(usize, preview::Msg),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::AllProjects(res) => {
            match res {
                Ok(ps) => { model.previews = ps
                    .iter()
                    .enumerate()
                    .map(|(i, u)|
                        preview::Model::from(
                            project::init(
                                project::Config::Entry(u.clone()),
                                &mut orders
                                    .proxy(move |p| Msg::Preview(i, preview::Msg::Project(p)))
                            )
                        )
                    )
                    .collect(); },
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
                Ok(ps) => { model.previews = ps
                    .iter()
                    .enumerate()
                    .map(|(i, u)|
                        preview::Model::from(
                            project::init(
                                project::Config::Entry(u.clone()),
                                &mut orders
                                    .proxy(move |p| Msg::Preview(i, preview::Msg::Project(p)))
                            )
                        )
                    )
                    .collect(); },
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
                msg.clone(),
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
            if let preview::Msg::Project(project::Msg::Deleted(_)) = msg {
                model.config.update(orders);
            }
        },
        Msg::OpenEditor => {
            model.editor = match model.config {
                Config::User(id) => {
                    Some(editor::init(editor::Config::User(id), &mut orders.proxy(Msg::Editor)))
                },
                _ => {
                    Some(editor::init(editor::Config::Empty, &mut orders.proxy(Msg::Editor)))
                },
            };
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
                    model.config.update(orders);
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
