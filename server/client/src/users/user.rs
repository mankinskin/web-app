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
    fetched::{
        self,
        Fetched,
        Query,
    },
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub user: fetched::Fetched<User>,
}
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    pub fn profile(&self) -> profile::Model {
        profile::Model::from(self.clone())
    }
    fn ready(user_id: Id<User>, user: User) -> Self {
        Self {
            user: Fetched::ready(
                      url::Url::parse("http://localhost:8000/api/users").unwrap(),
                      user,
                      Query::Id(user_id)
                      ),
        }
    }
    fn empty(query: Query<User>) -> Self {
        Self {
            user:
                Fetched::empty(
                    url::Url::parse("http://localhost:8000/api/users").unwrap(),
                    query,
                ),
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
impl From<Query<User>> for Model {
    fn from(query: Query<User>) -> Self {
        Self::empty(query)
    }
}
impl From<Id<User>> for Model {
    fn from(user_id: Id<User>) -> Self {
        Self::from(Query::Id(user_id))
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetched::Msg<User>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            model.user.update(msg, &mut orders.proxy(Msg::Fetch))
        },
    }
}
