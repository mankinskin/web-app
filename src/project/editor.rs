use crate::project::*;
use components::{
    editor::Msg as EditorMsg,
    Component,
    Edit,
    Editor,
};
use rql::Id;

#[derive(Clone, Default)]
pub struct Model {
    pub editor: Editor<Project>,
    pub user_id: Option<Id<User>>,
}
impl From<Id<User>> for Model {
    fn from(id: Id<User>) -> Self {
        Self {
            user_id: Some(id),
            ..Default::default()
        }
    }
}
impl From<Editor<Project>> for Model {
    fn from(editor: Editor<Project>) -> Self {
        Self {
            editor,
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Editor(EditorMsg<Project>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Editor(msg) => {
                match msg {
                    EditorMsg::Submit => {
                        match &mut self.editor {
                            Editor::New(new) => {
                                if let Some(id) = self.user_id {
                                    new.data.add_member(id);
                                }
                            }
                            Editor::Remote(_remote) => {}
                        };
                    }
                    _ => {}
                }
                Component::update(&mut self.editor, msg, &mut orders.proxy(Msg::Editor));
            }
        }
    }
}
impl Edit for Model {
    fn edit(&self) -> Node<Msg> {
        self.editor.edit().map_msg(Msg::Editor)
    }
}
