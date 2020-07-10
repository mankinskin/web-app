use rql::{
    *,
};
use crate::{
    root,
    config::{
        Config,
        Component,
        View,
    },
    user::*,
    project,
    remote,
};

#[derive(Clone)]
pub struct Model {
    pub entry: remote::Model<User>,
    pub projects: project::list::Model,
}
impl Config<Model> for Id<User> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            entry: Config::init(self.clone(), &mut orders.proxy(Msg::Entry)),
            projects: Config::init(self.clone(), &mut orders.proxy(Msg::ProjectList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<User> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            entry: remote::Model::from(self.clone()),
            projects: Config::init(self.id, &mut orders.proxy(Msg::ProjectList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}

#[derive(Clone)]
pub enum Msg {
    Entry(remote::Msg<User>),
    ProjectList(project::list::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg,
                    &mut orders.proxy(Msg::Entry)
                )
            },
            Msg::ProjectList(msg) => {
                self.projects.update(
                    msg,
                    &mut orders.proxy(Msg::ProjectList)
                )
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            self.entry.view()
                .map_msg(Msg::Entry),
            self.projects.view()
                .map_msg(Msg::ProjectList),
        ]
    }
}
