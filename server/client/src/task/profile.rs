use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
};
use rql::{
    Id,
};
use crate::{
    components::{
        Component,
        View,
        Config,
    },
    remote,
    task::{
        editor,
    },
    editor::{
        self as g_editor,
        Edit,
    },
    entry,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub entry: remote::Model<Task>,
    pub editor: Option<editor::Model>,
}
impl Model {
    fn refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(
            Msg::Entry(remote::Msg::Get)
        );
    }
}
impl Config<Model> for Id<Task> {
    fn init(self, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: Config::init(self.clone(), &mut orders.proxy(Msg::Entry)),
            editor: None,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            entry: remote::Model::from(entry),
            editor: None,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Entry(remote::Msg<Task>),
    Edit,
    Editor(editor::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg,
                    &mut orders.proxy(Msg::Entry),
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
                        "Edit Task"
                    ]
                } else { empty![] },
                self.entry.view().map_msg(Msg::Entry)
            ]
        }
    }
}
