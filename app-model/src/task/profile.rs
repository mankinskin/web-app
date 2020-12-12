use crate::{
    auth,
    task::*,
};
use components::{
    remote,
    Component,
    Edit,
    Editor,
    Init,
    Viewable,
};
use database_table::Entry;
use rql::Id;

#[derive(Debug, Clone)]
pub struct Model {
    pub entry: remote::Remote<Task>,
    pub editor: Option<editor::Model>,
}
impl Model {
    fn _refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(Msg::Entry(remote::Msg::Get));
    }
}
impl Init<Id<Task>> for Model {
    fn init(id: Id<Task>, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: Init::init(id, &mut orders.proxy(Msg::Entry)),
            editor: None,
        }
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            entry: remote::Remote::from(entry),
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
                Component::update(&mut self.entry, msg, &mut orders.proxy(Msg::Entry));
            }
            Msg::Edit => {
                self.editor = Some(Editor::from(self.entry.clone()).into());
            }
            Msg::Editor(msg) => {
                if let Some(editor) = &mut self.editor {
                    Component::update(editor, msg, &mut orders.proxy(Msg::Editor));
                }
                //match msg {
                //    editor::Msg::Editor(msg) =>
                //        match msg {
                //            g_editor::Msg::Cancel => {
                //                self.editor = None;
                //            },
                //            g_editor::Msg::Remote(msg) =>
                //                match msg {
                //                    remote::Msg::Entry(msg) =>
                //                        match msg {
                //                            entry::Msg::Updated(_) => {
                //                                self.refresh(orders);
                //                                self.editor = None;
                //                            },
                //                            _ => {}
                //                        },
                //                    _ => {}
                //                },
                //            _ => {},
                //        },
                //}
            }
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Msg> {
        if let Some(editor) = &self.editor {
            editor.edit().map_msg(Msg::Editor)
        } else {
            div![
                if let Some(_) = auth::session::get() {
                    button![ev(Ev::Click, |_| Msg::Edit), "Edit Task"]
                } else {
                    empty![]
                },
                self.entry.view().map_msg(Msg::Entry)
            ]
        }
    }
}
