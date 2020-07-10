use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
    project::*,
};
use crate::{
    config::{
        Component,
        View,
        Config,
    },
    root::{
        GMsg,
    },
    editor as g_editor,
    list,
    newdata,
    remote,
    entry,
    editor::{
        Edit,
    },
    task::{
        editor,
    },
};
use database::{
    Entry,
};
use rql::{
    *,
};
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model {
    project_id: Option<Id<Project>>,
    list: list::Model<Task>,
    editor: Option<editor::Model>,
}
impl Model {
    fn refresh(&self, orders: &mut impl Orders<Msg, GMsg>) {
        orders.send_msg(
            if let Some(id) = self.project_id {
                Msg::GetProjectTasks(id)
            } else {
                Msg::List(list::Msg::GetAll)
            }
        );
    }
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, GMsg>) {
        orders.send_msg(self);
    }
}
impl From<Vec<Entry<Task>>> for Model {
    fn from(entries: Vec<Entry<Task>>) -> Self {
        Self {
            list: list::Model::from(entries),
            ..Default::default()
        }
    }
}
impl Config<Model> for Id<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
        Model {
            project_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, GMsg>) {
        orders.send_msg(Msg::GetProjectTasks(self));
    }
}
#[derive(Clone)]
pub enum Msg {
    GetProjectTasks(Id<Project>),
    ProjectTasks(Result<Vec<Entry<Task>>, String>),

    List(list::Msg<Task>),

    New,
    Editor(editor::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::GetProjectTasks(id) => {
                orders.perform_cmd(
                    api::get_project_tasks(id)
                        .map(|res| Msg::ProjectTasks(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::ProjectTasks(res) => {
                match res {
                    Ok(entries) => self.list = list::Model::from(entries),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::List(msg) => {
                self.list.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::List)
                );
            },
            Msg::New => {
                self.editor = match self.project_id {
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
            h2!["Tasks"],
            if let Some(editor) = &self.editor {
                editor.edit().map_msg(Msg::Editor)
            } else {
                if let Some(_) = api::auth::get_session() {
                    button![
                        simple_ev(Ev::Click, Msg::New),
                        "New Task"
                    ]
                } else { empty![] }
            },
            self.list.view()
                .map_msg(Msg::List)
        ]
    }
}
