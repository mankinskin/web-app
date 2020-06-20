use crate::{
    users::*,
};

#[derive(Clone)]
pub struct Model {
    pub user: user::Model,
}
#[derive(Clone)]
pub enum Msg {
    User(user::Msg),
}
impl From<user::Model> for Model {
    fn from(user: user::Model) -> Self {
        Self {
            user,
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
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.user.user {
        Some(user) => {
            div![
                h1!["Profile"],
                p![user.name()],
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
