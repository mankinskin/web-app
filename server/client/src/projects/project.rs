use seed::{
    prelude::*,
};
use plans::{
    project::*,
};
use rql::{
    Id,
};
use crate::{
    config::*,
    root::{
        self,
        GMsg,
    },
    projects::*,
};
use database::{
    Entry,
};

impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Id<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            project_id: self.clone(),
            project: None,
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::Get(self));
    }
}
impl Config<Model> for Entry<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        let id = self.id().clone();
        let data = self.data().clone();
        Model {
            project_id: id.clone(),
            project: Some(data),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub struct Model {
    pub project_id: Id<Project>,
    pub project: Option<Project>,
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<Project>),
    GotProject(Result<Option<Entry<Project>>, String>),

    Delete,
    Deleted(Result<Option<Project>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_project(id)
                    .map(|res| Msg::GotProject(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::GotProject(res) => {
            match res {
                Ok(r) =>
                if let Some(entry) = r {
                    model.project_id = entry.id().clone();
                    model.project = Some(entry.data().clone());
                },
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Delete => {
            orders.perform_cmd(
                api::delete_project(model.project_id)
                .map(|res| Msg::Deleted(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Deleted(_res) => {
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!["Project"],
        match &model.project {
            Some(model) => {
                div![
                    p![model.name()],
                    button![
                        simple_ev(Ev::Click, Msg::Delete),
                        "Delete"
                    ],
                    //button![
                    //    simple_ev(Ev::Click, Msg::Edit),
                    //    "Edit"
                    //],
                ]
            },
            None => {
                div![
                    p!["Loading..."],
                ]
            },
        },
    ]
}
