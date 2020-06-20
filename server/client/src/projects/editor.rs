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
    fetch,
    fetch::Request,
};

#[derive(Clone)]
pub struct Model {
    pub project: Project,
}
impl Model {
    fn empty() -> Self {
        Self {
            project: Project::new(String::new()),
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeName(String),
    ChangeDescription(String),
    Create,
    Cancel,
    Fetch(fetch::Msg<Project>)
}
impl From<fetch::Msg<Project>> for Msg {
    fn from(msg: fetch::Msg<Project>) -> Self {
        Msg::Fetch(msg)
    }
}
impl Msg {
    pub fn post_project(project: &Project) -> Msg {
        Msg::Fetch(fetch::Msg::Request(Request::Post(project.clone())))
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
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/projects").unwrap(),
                            request,
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {
                    seed::log(error);
                },
            }
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
