use seed::{
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    root::{
        GMsg,
    },
    projects::*,
};

#[derive(Clone)]
pub struct Model {
    pub project: Project,
    pub project_id: Option<Id<Project>>,
    pub config: Config,
}
impl Model {
    fn empty() -> Self {
        Self {
            project: Project::new(String::new()),
            project_id: None,
            config: Config::Empty,
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Config {
    Empty,
    User(Id<User>),
}
impl From<Config> for Model {
    fn from(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}
impl Config {
    fn update(&self, _orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            _ => {}
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    config.update(orders);
    Model::from(config)
}
#[derive(Clone)]
pub enum Msg {
    ChangeName(String),
    ChangeDescription(String),
    Create,
    Cancel,
    CreateProject(Project),
    CreatedProject(Result<Id<Project>, String>),
}
impl Msg {
    pub fn post_project(project: &Project) -> Msg {
        Msg::CreateProject(project.clone())
    }
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
        Msg::CreateProject(p) => {
            let mut project = p;
            match model.config {
                Config::User(id) => { project.add_member(id); },
                _ => {},
            }
            orders.perform_cmd(
                api::post_project(project)
                    .map(|res| Msg::CreatedProject(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Create => {
            orders.send_msg(Msg::post_project(&model.project));
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
