use seed::{
    prelude::*,
};
use crate::{
    task::*,
};
use components::{
    Edit,
    Editor,
    editor::Msg as EditorMsg,
    Component,
};

#[derive(Clone, Default)]
pub struct Model {
    pub editor: Editor<Task>,
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
impl From<Editor<Task>> for Model {
    fn from(editor: Editor<Task>) -> Self {
        Self {
            editor,
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Editor(EditorMsg<Task>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Editor(msg) => {
                match msg {
                    EditorMsg::Submit => {
                        match &self.editor {
                            Editor::New(new) => {
                                let _task = new.data.clone();
                                if let Some(_id) = self.project_id {
                                    //orders.perform_cmd(
                                    //        api::project_create_subtask(id, task)
                                    //            .map(|res| Msg::Editor(
                                    //                    editor::Msg::New(
                                    //                        newdata::Msg::Posted(res.map_err(|e| format!("{:?}", e)))
                                    //                    )))
                                    //);
                                } else {
                                    //orders.perform_cmd(
                                    //        api::post_task(task)
                                    //            .map(|res| Msg::Editor(
                                    //                    editor::Msg::New(
                                    //                        newdata::Msg::Posted(res.map_err(|e| format!("{:?}", e)))
                                    //                    )))
                                    //);
                                }
                            },
                            Editor::Remote(_remote) => {
                                Component::update(
                                    &mut self.editor,
                                    msg,
                                    &mut orders.proxy(Msg::Editor),
                                );
                            }
                        };
                    },
                    _ => {
                        Component::update(
                            &mut self.editor,
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
