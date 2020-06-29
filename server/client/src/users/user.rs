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
#[derive(Clone)]
pub enum Config {
    Model(Model),
    UserId(Id<User>),
}
impl From<Id<User>> for Config {
    fn from(id: Id<User>) -> Self {
        Self::UserId(id)
    }
}
impl From<Model> for Config {
    fn from(model: Model) -> Self {
        Self::Model(model)
    }
}
impl Model {
    fn ready(id: Id<User>, user: User) -> Self {
        Self {
            user: Some(user),
            user_id: id,
        }
    }
    fn fetch(id: Id<User>) -> Self {
        Self {
            user: None,
            user_id: id,
        }
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::UserId(id) => {
            orders.send_msg(Msg::Get(id));
            Model::fetch(id)
        },
        Config::Model(model) => {
            model
        },
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
        Self::fetch(user_id)
    }
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<User>),
    User(Result<Option<User>, String>),
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
