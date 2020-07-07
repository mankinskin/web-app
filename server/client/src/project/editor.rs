use seed::{
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    config::{
        Config,
        Component,
        View,
    },
    root::{
        self,
        GMsg,
    },
    project::*,
};

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
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::ChangeName(n) => {
                self.project.set_name(n);
            },
            Msg::ChangeDescription(d) => {
                self.project.set_description(d);
            },
            Msg::Cancel => {},
            Msg::Submit => {
                let mut project = self.project.clone();
                if let Some(id) = self.user_id {
                    project.add_member(id);
                }
                orders.perform_cmd(
                    api::post_project(project)
                        .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::Created(res) => {
                match res {
                    Ok(id) => self.project_id = Some(id),
                    Err(e) => { seed::log(e); },
                }
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        form![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
            },
            if let Some(_) = self.project_id {
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
                    At::Value => self.project.name(),
                },
                input_ev(Ev::Input, Msg::ChangeName)
            ],
            label![
                "Description"
            ],
            textarea![
                attrs!{
                    At::Placeholder => "Description...",
                    At::Value => self.project.description(),
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
}
