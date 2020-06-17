use crate::{
    users::*,
};

#[derive(Clone, Default)]
pub struct Model {
    user: user::Model,
}
impl From<user::Model> for Model {
    fn from(user: user::Model) -> Self {
        Self {
            user,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    User(user::Msg),
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
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.user.user {
        Status::Ready(user) => {
            div![
                h1![user.name()],
                p!["Preview"],
            ]
        },
        Status::Loading => {
            div![
                h1!["Preview"],
                p!["Loading..."],
            ]
        },
        Status::Empty => {
            div![
                h1!["Empty Preview"],
            ]
        },
    }
}
