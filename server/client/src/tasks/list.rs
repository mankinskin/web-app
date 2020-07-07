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
    list,
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
    //editor: Option<editor::Model>,
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, GMsg>) {
        orders.send_msg(self);
    }
}
impl Config<Model> for Vec<Entry<Task>> {
    fn into_model(self, orders: &mut impl Orders<Msg, GMsg>) -> Model {
        Model {
            list: Config::init(self, &mut orders.proxy(Msg::List)),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, GMsg>) {
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
    GetAll,
    AllTasks(Result<Vec<Entry<Task>>, String>),

    GetProjectTasks(Id<Project>),
    ProjectTasks(Result<Vec<Entry<Task>>, String>),

    List(list::Msg<Task>),

    //OpenEditor,
    //Editor(editor::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    api::get_tasks()
                        .map(|res| Msg::AllTasks(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::AllTasks(res) => {
                match res {
                    Ok(entries) =>
                        self.list = Config::init(
                            entries,
                            &mut orders.proxy(Msg::List)
                        ),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::GetProjectTasks(id) => {
                orders.perform_cmd(
                    api::get_project_tasks(id)
                        .map(|res| Msg::ProjectTasks(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::ProjectTasks(res) => {
                match res {
                    Ok(entries) =>
                        self.list = Config::init(
                            entries,
                            &mut orders.proxy(Msg::List)
                        ),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::List(msg) => {
                self.list.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::List)
                );
            },
            //Msg::OpenEditor => {
            //    model.editor = match model.project_id {
            //        Some(id) => {
            //            Some(Config::init(id, &mut orders.proxy(Msg::Editor)))
            //        },
            //        None => {
            //            Some(editor::Model::default())
            //        },
            //    };
            //},
            //Msg::Editor(msg) => {
            //    if let Some(editor) = &mut model.editor {
            //        editor::update(
            //            msg.clone(),
            //            editor,
            //            &mut orders.proxy(Msg::Editor)
            //        );
            //    }
            //    match msg {
            //        editor::Msg::Cancel => {
            //            model.editor = None;
            //        },
            //        editor::Msg::Created(_) => {
            //            orders.send_msg(
            //                if let Some(id) = model.project_id {
            //                    Msg::GetProjectTasks(id)
            //                } else {
            //                    Msg::GetAll
            //                }
            //            );
            //        },
            //        _ => {},
            //    }
            //},
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            //if let Some(model) = &model.editor {
            //    editor::view(&model).map_msg(Msg::Editor)
            //} else {
            //    if let Some(_) = api::auth::get_session() {
            //        button![
            //            simple_ev(Ev::Click, Msg::OpenEditor),
            //            "New Task"
            //        ]
            //    } else { empty![] }
            //},
            self.list.view()
                .map_msg(Msg::List)
        ]
    }
}
