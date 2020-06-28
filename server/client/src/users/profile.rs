use rql::{
    *,
};
use crate::{
    users::*,
    projects,
};

#[derive(Clone)]
pub struct Model {
    pub user: user::Model,
    pub projects: projects::Model,
}
#[derive(Clone)]
pub enum Config {
    UserId(Id<User>),
    User(user::Model),
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::UserId(id) => {
            Model {
                user: user::init(user::Config::UserId(id), &mut orders.proxy(Msg::User)),
                projects: projects::init(projects::Config::User(id), &mut orders.proxy(Msg::Projects)),
            }
        },
        Config::User(model) => {
            Model {
                user: model.clone(),
                projects: projects::init(projects::Config::User(model.user_id), &mut orders.proxy(Msg::Projects)),
            }
        },
    }
}
#[derive(Clone)]
pub enum Msg {
    User(user::Msg),
    Projects(projects::Msg),
}
impl From<user::Model> for Config {
    fn from(model: user::Model) -> Self {
        Self::User(model)
    }
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
        Msg::Projects(msg) => {
            projects::update(
                msg,
                &mut model.projects,
                &mut orders.proxy(Msg::Projects)
            )
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.user.user {
        Some(user) => {
            div![
                h1!["Profile"],
                p![user.name()],
                projects::view(&model.projects)
                    .map_msg(Msg::Projects),
            ]
        },
        None => {
            div![
                h1!["Profile"],
                p!["Loading..."],
            ]
        },
    }
}
