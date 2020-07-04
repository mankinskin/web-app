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
    config::*,
    root::{
        self,
        GMsg,
    },
    projects::{
        preview,
        editor,
        project,
    },
};
use database::{
    Entry,
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(self);
    }
}
impl Config<Model> for Vec<Entry<Project>> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            previews: init_previews(self, orders),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Id<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            user_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::GetUserProjects(self));
    }
}
fn init_previews(entries: Vec<Entry<Project>>, orders: &mut impl Orders<Msg, GMsg>) -> Vec<preview::Model> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)|
            Config::init(
                entry.clone(),
                &mut orders
                    .proxy(move |msg| Msg::Preview(i, msg))
            )
        )
        .collect()
}
#[derive(Clone, Default)]
pub struct Model {
    user_id: Option<Id<User>>,
    previews: Vec<preview::Model>,
    editor: Option<editor::Model>,
}
#[derive(Clone)]
pub enum Msg {
    GetAll,
    AllProjects(Result<Vec<Entry<Project>>, String>),

    GetUserProjects(Id<User>),
    UserProjects(Result<Vec<Entry<Project>>, String>),

    Preview(usize, preview::Msg),

    OpenEditor,
    Editor(editor::Msg),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::GetAll => {
            orders.perform_cmd(
                api::get_projects()
                    .map(|res| Msg::AllProjects(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::AllProjects(res) => {
            match res {
                Ok(ps) => model.previews = init_previews(ps, orders),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::GetUserProjects(id) => {
            orders.perform_cmd(
                api::get_user_projects(id)
                .map(|res| Msg::UserProjects(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::UserProjects(res) => {
            match res {
                Ok(ps) => model.previews = init_previews(ps, orders),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Preview(index, msg) => {
            preview::update(
                msg.clone(),
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
            if let preview::Msg::Project(project::Msg::Deleted(_)) = msg {
                model.previews.remove(index);
            }
        },
        Msg::OpenEditor => {
            model.editor = match model.user_id {
                Some(id) => {
                    Some(Config::init(id, &mut orders.proxy(Msg::Editor)))
                },
                None => {
                    Some(editor::Model::default())
                },
            };
        },
        Msg::Editor(msg) => {
            if let Some(editor) = &mut model.editor {
                editor::update(
                    msg.clone(),
                    editor,
                    &mut orders.proxy(Msg::Editor)
                );
            }
            match msg {
                editor::Msg::Cancel => {
                    model.editor = None;
                },
                editor::Msg::Created(_) => {
                    orders.send_msg(
                        if let Some(id) = model.user_id {
                            Msg::GetUserProjects(id)
                        } else {
                            Msg::GetAll
                        }
                    );
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
