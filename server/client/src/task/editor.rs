use seed::{
    prelude::*,
};
use plans::{
    task::*,
    project::*,
};
use crate::{
    config::*,
    root::{
        GMsg,
    },
    task::{*},
    editor::{
        self,
        Edit,
    },
    newdata,
};

#[derive(Clone, Default)]
pub struct Model {
    pub editor: editor::Model<Task>,
    pub project_id: Option<Id<Project>>,
}
impl From<Id<Project>> for Model {
    fn from(id: Id<Project>) -> Self {
        Model {
            project_id: Some(id),
            ..Default::default()
        }
    }
}
impl<T: Into<editor::Model<Task>>> From<T> for Model {
    fn from(t: T) -> Self {
        Self {
            editor: t.into(),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Editor(editor::Msg<Task>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Editor(msg) => {
                match msg {
                    editor::Msg::Submit => {
                        match &self.editor {
                            editor::Model::New(new) => {
                                let task = new.data.clone();
                                if let Some(id) = self.project_id {
                                    orders.perform_cmd(
                                            api::project_create_subtask(id, task)
                                                .map(|res| Msg::Editor(
                                                        editor::Msg::New(
                                                            newdata::Msg::Posted(res.map_err(|e| format!("{:?}", e)))
                                                        )))
                                    );
                                } else {
                                    orders.perform_cmd(
                                            api::post_task(task)
                                                .map(|res| Msg::Editor(
                                                        editor::Msg::New(
                                                            newdata::Msg::Posted(res.map_err(|e| format!("{:?}", e)))
                                                        )))
                                    );
                                }
                            },
                            editor::Model::Remote(_remote) => {
                                self.editor.update(
                                    msg,
                                    &mut orders.proxy(Msg::Editor),
                                );
                            }
                        };
                    },
                    _ => {
                        self.editor.update(
                            msg,
                            &mut orders.proxy(Msg::Editor),
                        );
                    }
                }
            },
        }
    }
}
impl Edit for Model {
    fn edit(&self) -> Node<Msg> {
        self.editor.edit().map_msg(Msg::Editor)
    }
}
