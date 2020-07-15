use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
};
use rql::{
    Id,
};
use crate::{
    config::{
        Component,
        View,
        Config,
    },
    task,
    remote,
    entry,
    project::{
        editor,
    },
    editor::{
        self as g_editor,
        Edit,
    },
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub entry: remote::Model<Project>,
    pub tasks: task::list::Model,
    pub editor: Option<editor::Model>,
}
impl Model {
    fn refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(
            Msg::Entry(remote::Msg::Get)
        );
    }
}
impl Config<Model> for Id<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: Config::init(self.clone(), &mut orders.proxy(Msg::Entry)),
            tasks: Config::init(self, &mut orders.proxy(Msg::TaskList)),
            editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg>) {
    }
}
impl Config<Model> for Entry<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: remote::Model::from(self.clone()),
            tasks: Config::init(self.id, &mut orders.proxy(Msg::TaskList)),
            editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg>) {
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Entry(remote::Msg<Project>),
    TaskList(task::list::Msg),
    Editor(editor::Msg),
    Edit,
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg,
                    &mut orders.proxy(Msg::Entry)
                );
            },
            Msg::TaskList(msg) => {
                self.tasks.update(
                    msg,
                    &mut orders.proxy(Msg::TaskList)
                );
            },
            Msg::Edit => {
                self.editor = Some(editor::Model::from(self.entry.clone()));
            },
            Msg::Editor(msg) => {
                if let Some(editor) = &mut self.editor {
                    editor.update(
                        msg.clone(),
                        &mut orders.proxy(Msg::Editor)
                    );
                }
                match msg {
                    editor::Msg::Editor(msg) =>
                        match msg {
                            g_editor::Msg::Cancel => {
                                self.editor = None;
                            },
                            g_editor::Msg::Remote(msg) =>
                                match msg {
                                    remote::Msg::Entry(msg) =>
                                        match msg {
                                            entry::Msg::Updated(_) => {
                                                self.refresh(orders);
                                                self.editor = None;
                                            },
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
        if let Some(editor) = &self.editor {
            editor.edit().map_msg(Msg::Editor)
        } else {
            div![
                if let Some(_) = api::auth::get_session() {
                    button![
                        ev(Ev::Click, |_| Msg::Edit),
                        "Edit Project"
                    ]
                } else { empty![] },
                self.entry.view().map_msg(Msg::Entry),
                self.tasks.view().map_msg(Msg::TaskList),
            ]
        }
    }
}
