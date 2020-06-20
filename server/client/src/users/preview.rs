use crate::{
    page,
    users::*,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub user: user::Model,
}
impl From<user::Model> for Model {
    fn from(user: user::Model) -> Self {
        Self {
            user,
        }
    }
}
impl From<&Entry<User>> for Model {
    fn from(entry: &Entry<User>) -> Self {
        Self {
            user: user::Model::from(entry),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    User(user::Msg),
    Open,
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
        Msg::Open => {
            page::go_to(profile::Model::from(model.user.clone()), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.user.user {
        Some(user) => {
            div![
                a![
                    attrs!{
                        At::Href => "";
                    },
                    user.name(),
                    simple_ev(Ev::Click, Msg::Open),
                ],
                p!["Preview"],
            ]
        },
        None => {
            div![
                h1!["Preview"],
                p!["Loading..."],
            ]
        },
    }
}
