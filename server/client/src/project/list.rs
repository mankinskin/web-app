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
    config::{
        Config,
        Component,
        View,
    },
    list,
    project::editor,
    editor::{
        self as g_editor,
        Edit,
    },
    newdata,
    remote,
    entry,
};
use database::{
    Entry,
};
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model {
    user_id: Option<Id<User>>,
    list: list::Model<Project>,
    editor: Option<editor::Model>,
}
impl Model {
    fn refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(
            if let Some(id) = self.user_id {
                Msg::GetUserProjects(id)
            } else {
                Msg::List(list::Msg::GetAll)
            }
        );
    }
}
impl Config<Model> for list::Msg<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(Msg::List(self));
    }
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(self);
    }
}
impl From<Vec<Entry<Project>>> for Model {
    fn from(entries: Vec<Entry<Project>>) -> Self {
        Self {
            list: list::Model::from(entries),
            ..Default::default()
        }
    }
}
impl Config<Model> for Id<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg>) -> Model {
        Model {
            user_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(Msg::GetUserProjects(self));
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetUserProjects(Id<User>),
    UserProjects(Result<Vec<Entry<Project>>, String>),

    List(list::Msg<Project>),

    New,
    Editor(editor::Msg),
}

impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::GetUserProjects(id) => {
                orders.perform_cmd(
                    api::get_user_projects(id)
                    .map(|res| Msg::UserProjects(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::UserProjects(res) => {
                match res {
                    Ok(entries) => self.list = list::Model::from(entries),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::List(msg) => {
                self.list.update(
                    msg,
                    &mut orders.proxy(Msg::List)
                );
            },
            Msg::New => {
                self.editor = match self.user_id {
                    Some(id) => {
                        Some(editor::Model::from(id))
                    },
                    None => {
                        Some(editor::Model::default())
                    },
                };
            },
            Msg::Editor(msg) => {
                if let Some(editor) = &mut self.editor {
                    editor.update(
                        msg.clone(),
                        &mut orders.proxy(Msg::Editor)
                    );
                }
                // TODO improve inter component messaging
                match msg {
                    editor::Msg::Editor(msg) =>
                        match msg {
                            g_editor::Msg::Cancel => {
                                self.editor = None;
                            },
                            g_editor::Msg::New(msg) =>
                                match msg {
                                    newdata::Msg::Posted(_) =>
                                        self.refresh(orders),
                                    _ => {}
                                },
                            g_editor::Msg::Remote(msg) =>
                                match msg {
                                    remote::Msg::Entry(msg) =>
                                        match msg {
                                            entry::Msg::Updated(_) =>
                                                self.refresh(orders),
                                            _ => {}
                                        },
                                    _ => {}
                                },
                            _ => {},
                        },
                }
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            h2!["Projects"],
            if let Some(editor) = &self.editor {
                editor.edit().map_msg(Msg::Editor)
            } else {
                if let Some(_) = api::auth::get_session() {
                    button![
                        ev(Ev::Click, |_| Msg::New),
                        "New Project"
                    ]
                } else { empty![] }
            },
            self.list.view().map_msg(Msg::List)
        ]
    }
}
