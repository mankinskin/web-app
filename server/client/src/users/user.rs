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
    fetch::{
        self,
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
    Fetch(fetch::Msg<User>),
}
impl Msg {
    pub fn fetch_user(id: Id<User>) -> Msg {
        Msg::Fetch(fetch::Msg::Request(fetch::Request::Get(Query::Id(id))))
    }
}
impl From<fetch::Msg<User>> for Msg {
    fn from(msg: fetch::Msg<User>) -> Self {
        Msg::Fetch(msg)
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/users").unwrap(),
                            request,
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        fetch::Response::Get(data) => {
                            model.user = Some(data);
                        },
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {
                    seed::log(error);
                },
            }
        },
    }
}
