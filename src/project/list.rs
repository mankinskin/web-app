use seed::{
    *,
    prelude::*,
};
use components::{
    Init,
    Component,
    Viewable,
    Edit,
    list,
};
use rql::{
    *,
};
use crate::{
    auth,
    project::*,
};
use database_table::{
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
    fn _refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(
            if let Some(id) = self.user_id {
                Msg::GetUserProjects(id)
            } else {
                Msg::List(list::Msg::GetAll)
            }
        );
    }
}
impl Init<list::Msg<Project>> for Model {
    fn init(msg: list::Msg<Project>, orders: &mut impl Orders<Msg>) -> Model {
        orders.send_msg(Msg::List(msg));
        Model::default()
    }
}
impl Init<Msg> for Model {
    fn init(msg: Msg, orders: &mut impl Orders<Msg>) -> Model {
        orders.send_msg(msg);
        Model::default()
    }
}
impl Init<Id<User>> for Model {
    fn init(id: Id<User>, orders: &mut impl Orders<Msg>) -> Model {
        orders.send_msg(Msg::GetUserProjects(id));
        Model {
            user_id: Some(id),
            ..Default::default()
        }
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
            Msg::GetUserProjects(_id) => {
                //orders.perform_cmd(
                //    api::get_user_projects(id)
                //    .map(|res| Msg::UserProjects(res.map_err(|e| format!("{:?}", e))))
                //);
            },
            Msg::UserProjects(res) => {
                match res {
                    Ok(entries) => self.list = list::Model::from(entries),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::List(msg) => {
                Component::update(
                    &mut self.list,
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
                    Component::update(
                        editor,
                        msg.clone(),
                        &mut orders.proxy(Msg::Editor)
                    );
                }
                // TODO improve inter component messaging
                //match msg {
                //    editor::Msg::Editor(msg) =>
                //        match msg {
                //            g_editor::Msg::Cancel => {
                //                self.editor = None;
                //            },
                //            g_editor::Msg::New(msg) =>
                //                match msg {
                //                    newdata::Msg::Posted(_) =>
                //                        self.refresh(orders),
                //                    _ => {}
                //                },
                //            g_editor::Msg::Remote(msg) =>
                //                match msg {
                //                    remote::Msg::Entry(msg) =>
                //                        match msg {
                //                            entry::Msg::Updated(_) =>
                //                                self.refresh(orders),
                //                            _ => {}
                //                        },
                //                    _ => {}
                //                },
                //            _ => {},
                //        },
                //}
            },
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Msg> {
        div![
            h2!["Projects"],
            if let Some(editor) = &self.editor {
                editor.edit().map_msg(Msg::Editor)
            } else {
                if let Some(_) = auth::session::get() {
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
