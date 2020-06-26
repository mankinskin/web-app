use seed::{
    prelude::*,
};
use plans::{
    user::*,
};
use rql::{
    Id,
};
use crate::{
    users::*,
    root::{
        GMsg,
    },
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub user_id: Id<User>,
    pub user: Option<User>,
}
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    pub fn profile(&self) -> profile::Model {
        profile::Model::from(self.clone())
    }
    fn ready(id: Id<User>, user: User) -> Self {
        Self {
            user_id: id,
            user: Some(user),
        }
    }
    fn fetch_id(id: Id<User>) -> Self {
        Self {
            user: None,
            user_id: id,
        }
    }
}
impl From<&Entry<User>> for Model {
    fn from(entry: &Entry<User>) -> Self {
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Entry<User>> for Model {
    fn from(entry: Entry<User>) -> Self {
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Id<User>> for Model {
    fn from(user_id: Id<User>) -> Self {
        Self::fetch_id(user_id)
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<User>),
    User(Result<Option<User>, String>),
}
impl Msg {
    pub fn fetch_user(id: Id<User>) -> Msg {
        Msg::Get(id)
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::User(res) => {
            match res {
                Ok(u) => model.user = u,
                Err(e) => { seed::log(e); },
            }
        },
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_user(id)
                    .map(|res| Msg::User(res.map_err(|e| format!("{:?}", e))))
            );
        },
    }
}
