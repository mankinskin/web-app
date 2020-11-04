use crate::{
    auth,
    project::*,
    task,
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

#[derive(Clone)]
pub struct Model {
    pub entry: remote::Remote<Project>,
    pub tasks: task::list::Model,
    pub editor: Option<editor::Model>,
}
impl Model {
    fn _refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(Msg::Entry(remote::Msg::Get));
    }
}
impl Init<Id<Project>> for Model {
    fn init(id: Id<Project>, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: Init::init(id.clone(), &mut orders.proxy(Msg::Entry)),
            tasks: Init::init(id, &mut orders.proxy(Msg::TaskList)),
            editor: None,
        }
    }
}
impl Init<Entry<Project>> for Model {
    fn init(entry: Entry<Project>, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: remote::Remote::from(entry.clone()),
            tasks: Init::init(entry.id, &mut orders.proxy(Msg::TaskList)),
            editor: None,
        }
    }
}
#[derive(Debug)]
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
                Component::update(&mut self.entry, msg, &mut orders.proxy(Msg::Entry));
            }
            Msg::TaskList(msg) => {
                Component::update(&mut self.tasks, msg, &mut orders.proxy(Msg::TaskList));
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
                    button![ev(Ev::Click, |_| Msg::Edit), "Edit Project"]
                } else {
                    empty![]
                },
                self.entry.view().map_msg(Msg::Entry),
                self.tasks.view().map_msg(Msg::TaskList),
            ]
        }
    }
}
