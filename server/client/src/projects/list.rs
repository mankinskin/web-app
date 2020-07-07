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
    root::{
        self,
        GMsg,
    },
    list,
};
use database::{
    Entry,
};
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model {
    user_id: Option<Id<User>>,
    list: list::Model<Project>,
    //editor: Option<editor::Model>,
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
            list: Config::init(self, &mut orders.proxy(Msg::List)),
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
#[derive(Clone)]
pub enum Msg {
    GetAll,
    AllProjects(Result<Vec<Entry<Project>>, String>),

    GetUserProjects(Id<User>),
    UserProjects(Result<Vec<Entry<Project>>, String>),

    List(list::Msg<Project>),

    //OpenEditor,
    //Editor(editor::Msg),
}

impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    api::get_projects()
                        .map(|res| Msg::AllProjects(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::AllProjects(res) => {
                match res {
                    Ok(entries) => self.list = Config::init(entries, &mut orders.proxy(Msg::List)),
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
                    Ok(entries) => self.list = Config::init(entries, &mut orders.proxy(Msg::List)),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::List(msg) => {
                self.list.update(
                    msg,
                    &mut orders.proxy(Msg::List)
                );
            },
            //Msg::OpenEditor => {
            //    self.editor = match self.user_id {
            //        Some(id) => {
            //            Some(Config::init(id, &mut orders.proxy(Msg::Editor)))
            //        },
            //        None => {
            //            Some(editor::Model::default())
            //        },
            //    };
            //},
            //Msg::Editor(msg) => {
            //    if let Some(editor) = &mut self.editor {
            //        editor.update(
            //            msg.clone(),
            //            &mut orders.proxy(Msg::Editor)
            //        );
            //    }
            //    match msg {
            //        editor::Msg::Cancel => {
            //            self.editor = None;
            //        },
            //        editor::Msg::Created(_) => {
            //            orders.send_msg(
            //                if let Some(id) = self.user_id {
            //                    Msg::GetUserProjects(id)
            //                } else {
            //                    Msg::GetAll
            //                }
            //            );
            //        },
            //        _ => {},
            //    }
            //},
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            //if let Some(editor) = &self.editor {
            //    editor.view().map_msg(Msg::Editor)
            //} else {
            //    if let Some(_) = api::auth::get_session() {
            //        button![
            //            simple_ev(Ev::Click, Msg::OpenEditor),
            //            "New Project"
            //        ]
            //    } else { empty![] }
            //},
            self.list.view().map_msg(Msg::List)
        ]
    }
}
