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
            project_id: None,
            project: Default::default(),
            user_id: Some(self),
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
            user_id: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeName(String),
    ChangeDescription(String),
    Create,
    Cancel,
    CreatedProject(Result<Id<Project>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeName(n) => {
            model.project.set_name(n);
        },
        Msg::ChangeDescription(d) => {
            model.project.set_description(d);
        },
        Msg::CreatedProject(res) => {
            match res {
                Ok(id) => model.project_id = Some(id),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Create => {
            let mut project = model.project.clone();
            if let Some(id) = model.user_id {
                project.add_member(id);
            }
            orders.perform_cmd(
                api::post_project(project)
                    .map(|res| Msg::CreatedProject(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Cancel => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    form![
        h1!["New Project"],
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
        // Cancel Button
        button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
        // Create Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Create"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Create
        }),
    ]
}
