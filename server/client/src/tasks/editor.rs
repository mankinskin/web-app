use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use crate::{
    config::*,
    root::{
        self,
        GMsg,
    },
    tasks::{*},
};

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone, Default)]
pub struct Model {
    pub task: Task,
    pub task_id: Option<Id<Task>>,
    pub project_id: Option<Id<Project>>,
}
impl Config<Model> for Id<Project> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: Default::default(),
            task_id: None,
            project_id: Some(self),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for task::Model {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: self.task.unwrap_or(Default::default()),
            task_id: Some(self.task_id),
            project_id: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task_id: Some(entry.id().clone()),
            task: entry.data().clone(),
            project_id: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Submit,
    Cancel,
}
pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::Submit => {
        },
        Msg::Cancel => {},
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
        if let Some(_) = model.task_id {
            h1!["Edit Task"]
        } else {
            h1!["New Task"]
        },
        label![
            "Title"
        ],
        input![
            attrs!{
                At::Placeholder => "Title",
                At::Value => model.task.title(),
            },
            input_ev(Ev::Input, Msg::ChangeTitle)
        ],
        label![
            "Description"
        ],
        textarea![
            attrs!{
                At::Placeholder => "Description...",
                At::Value => model.task.description(),
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
