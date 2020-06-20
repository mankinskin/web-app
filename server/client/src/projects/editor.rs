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
}
pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeName(n) => {
            model.project.set_name(n);
        },
        Msg::ChangeDescription(d) => {
            model.project.set_description(d);
        },
        Msg::Create => {

        },
        Msg::Cancel => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
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
