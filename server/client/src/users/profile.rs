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
pub enum Msg {
    User(user::Msg),
    Projects(projects::Msg),
}
impl From<user::Model> for Model {
    fn from(user: user::Model) -> Self {
        Self {
            user,
            projects: projects::Model::empty(),
        }
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
