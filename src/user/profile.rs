use super::*;
use crate::project;
use components::{
    remote,
    Component,
    Init,
    Viewable,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    user: User,
}
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self { user }
    }
}

#[derive(Clone)]
pub struct Model {
    pub entry: remote::Remote<User>,
    pub projects: project::list::Model,
}
impl Init<Id<User>> for Model {
    fn init(id: Id<User>, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: Init::init(id.clone(), &mut orders.proxy(Msg::Entry)),
            projects: Init::init(id.clone(), &mut orders.proxy(Msg::ProjectList)),
        }
    }
}
impl Init<Entry<User>> for Model {
    fn init(entry: Entry<User>, orders: &mut impl Orders<Msg>) -> Model {
        Model {
            entry: remote::Remote::from(entry.clone()),
            projects: Init::init(entry.id, &mut orders.proxy(Msg::ProjectList)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    Entry(remote::Msg<User>),
    ProjectList(project::list::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Entry(msg) => {
                Component::update(&mut self.entry, msg, &mut orders.proxy(Msg::Entry))
            }
            Msg::ProjectList(msg) => {
                Component::update(&mut self.projects, msg, &mut orders.proxy(Msg::ProjectList))
            }
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Msg> {
        div![
            self.entry.view().map_msg(Msg::Entry),
            self.projects.view().map_msg(Msg::ProjectList),
        ]
    }
}
