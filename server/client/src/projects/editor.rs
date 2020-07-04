use seed::{
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    config::*,
    root::{
        self,
        GMsg,
    },
    projects::*,
};

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone, Default)]
pub struct Model {
    pub project: Project,
    pub project_id: Option<Id<Project>>,
    pub user_id: Option<Id<User>>,
}
impl Config<Model> for Id<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            user_id: Some(self),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for project::Model {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            project: self.project.unwrap_or(Default::default()),
            project_id: Some(self.project_id),
            ..Default::default()
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl From<Entry<Project>> for Model {
    fn from(entry: Entry<Project>) -> Self {
        Self {
            project_id: Some(entry.id().clone()),
            project: entry.data().clone(),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeName(String),
    ChangeDescription(String),
    Cancel,
    Submit,
    Created(Result<Id<Project>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeName(n) => {
            model.project.set_name(n);
        },
        Msg::ChangeDescription(d) => {
            model.project.set_description(d);
        },
        Msg::Cancel => {},
        Msg::Submit => {
            let mut project = model.project.clone();
            if let Some(id) = model.user_id {
                project.add_member(id);
            }
            orders.perform_cmd(
                api::post_project(project)
                    .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Created(res) => {
            match res {
                Ok(id) => model.project_id = Some(id),
                Err(e) => { seed::log(e); },
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    form![
        style!{
            St::Display => "grid",
            St::GridTemplateColumns => "1fr",
            St::GridGap => "10px",
            St::MaxWidth => "20%",
        },
        if let Some(_) = model.project_id {
            h1!["Edit Project"]
        } else {
            h1!["New Project"]
        },
        label![
            "Name"
        ],
        input![
            attrs!{
                At::Placeholder => "Name",
                At::Value => model.project.name(),
            },
            input_ev(Ev::Input, Msg::ChangeName)
        ],
        label![
            "Description"
        ],
        textarea![
            attrs!{
                At::Placeholder => "Description...",
                At::Value => model.project.description(),
            },
            input_ev(Ev::Input, Msg::ChangeDescription)
        ],
        // Submit Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Create"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Submit
        }),
        // Cancel Button
        button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
    ]
}
