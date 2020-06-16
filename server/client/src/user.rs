use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use rql::{
    *,
};

#[derive(Clone, Default)]
pub struct Model {
    user_id: Id<User>,
    user: Option<User>,
}
impl From<Id<User>> for Model {
    fn from(user_id: Id<User>) -> Self {
        Self {
            user_id,
            user: None,
        }
    }
}

#[derive(Clone)]
pub enum Msg {

}
pub fn update(_msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg>) {
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(user) = &model.user {
        div![
            p![user.name()]
        ]
    } else {
        div![
            p!["User not found"]
        ]
    }
}
