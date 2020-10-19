use crate::{
    auth,
    task::editor,
    task::*,
};
use components::{
    list,
    Component,
    Edit,
    Init,
    Viewable,
};
use database_table::Entry;
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model {
    project_id: Option<Id<Project>>,
    list: list::Model<Task>,
    editor: Option<editor::Model>,
}
impl Model {
    fn _refresh(&self, orders: &mut impl Orders<Msg>) {
        orders.send_msg(if let Some(id) = self.project_id {
            Msg::GetProjectTasks(id)
        } else {
            Msg::List(list::Msg::GetAll)
        });
    }
}
impl Init<Msg> for Model {
    fn init(msg: Msg, orders: &mut impl Orders<Msg>) -> Model {
        orders.send_msg(msg);
        Model::default()
    }
}
impl Init<Id<Project>> for Model {
    fn init(id: Id<Project>, orders: &mut impl Orders<Msg>) -> Model {
        orders.send_msg(Msg::GetProjectTasks(id));
        Model {
            project_id: Some(id),
            ..Default::default()
        }
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
#[derive(Clone, Debug)]
pub enum Msg {
    GetProjectTasks(Id<Project>),
    ProjectTasks(Result<Vec<Entry<Task>>, String>),

    List(list::Msg<Task>),

    New,
    Editor(editor::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::GetProjectTasks(_id) => {
                //orders.perform_cmd(
                //    api::get_project_tasks(id)
                //        .map(|res| Msg::ProjectTasks(res.map_err(|e| format!("{:?}", e))))
                //);
            }
            Msg::ProjectTasks(res) => {
                match res {
                    Ok(entries) => self.list = list::Model::from(entries),
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::List(msg) => {
                Component::update(&mut self.list, msg.clone(), &mut orders.proxy(Msg::List));
            }
            Msg::New => {
                self.editor = match self.project_id {
                    Some(id) => Some(editor::Model::from(id)),
                    None => Some(editor::Model::default()),
                };
            }
            Msg::Editor(msg) => {
                if let Some(editor) = &mut self.editor {
                    Component::update(editor, msg.clone(), &mut orders.proxy(Msg::Editor));
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
            }
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Msg> {
        div![
            h2!["Tasks"],
            if let Some(editor) = &self.editor {
                editor.edit().map_msg(Msg::Editor)
            } else {
                if let Some(_) = auth::session::get() {
                    button![ev(Ev::Click, |_| Msg::New), "New Task"]
                } else {
                    empty![]
                }
            },
            self.list.view().map_msg(Msg::List)
        ]
    }
}
