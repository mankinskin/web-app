use rql::{
    *,
};
use crate::{
    root,
    config::*,
    users::*,
    projects,
};

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone, Default)]
pub struct Model {
    pub user: user::Model,
    pub projects: projects::list::Model,
}
impl Config<Model> for Id<User> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            user: Config::init(self.clone(), &mut orders.proxy(Msg::User)),
            projects: Config::init(self, &mut orders.proxy(Msg::ProjectList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<User> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        let id = self.id().clone();
        Model {
            user: Config::init(self, &mut orders.proxy(Msg::User)),
            projects: Config::init(id, &mut orders.proxy(Msg::ProjectList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}

#[derive(Clone)]
pub enum Msg {
    User(user::Msg),
    ProjectList(projects::list::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::User(msg) => {
            user::update(
                msg,
                &mut model.user,
                &mut orders.proxy(Msg::User)
            )
        },
        Msg::ProjectList(msg) => {
            projects::list::update(
                msg,
                &mut model.projects,
                &mut orders.proxy(Msg::ProjectList)
            )
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        user::view(&model.user)
            .map_msg(Msg::User),
        projects::list::view(&model.projects)
            .map_msg(Msg::ProjectList),
    ]
}
