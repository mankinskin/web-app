use seed::{
    prelude::*,
};
use plans::{
    project::*,
    user::*,
};
use crate::{
    config::{
        Component,
    },
    editor::{
        self,
        Edit,
    },
};
use rql::{
    Id,
};

#[derive(Clone, Default)]
pub struct Model {
    pub editor: editor::Model<Project>,
    pub user_id: Option<Id<User>>,
}
impl From<Id<User>> for Model {
    fn from(id: Id<User>) -> Self {
        Model {
            user_id: Some(id),
            ..Default::default()
        }
    }
}
impl<T: Into<editor::Model<Project>>> From<T> for Model {
    fn from(t: T) -> Self {
        Self {
            editor: t.into(),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Editor(editor::Msg<Project>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Editor(msg) => {
                match msg {
                    editor::Msg::Submit => {
                        match &mut self.editor {
                            editor::Model::New(new) =>
                                if let Some(id) = self.user_id {
                                    new.data.add_member(id);
                                },
                            editor::Model::Remote(_remote) => {}
                        };
                    },
                    _ => {}
                }
                self.editor.update(
                    msg,
                    &mut orders.proxy(Msg::Editor),
                );
            },
        }
    }
}
impl Edit for Model {
    fn edit(&self) -> Node<Msg> {
        self.editor.edit().map_msg(Msg::Editor)
    }
}
