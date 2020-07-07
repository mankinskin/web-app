use plans::{
    user::*,
};
use crate::{
    root::{
        GMsg,
    },
    config::{
        Component,
        View,
        Child,
    },
    entry::{
        self,
        TableItem,
    },
    preview::{self, *},
    route::{
        self,
    },
};
use rql::{
    *,
};
use database::{
    Entry,
};
use seed::{
    *,
    prelude::*,
};
use std::result::Result;
use async_trait::async_trait;

pub mod profile;
//pub mod list;

#[derive(Clone)]
pub enum Msg {
    Entry(Box<entry::Msg<User>>),
}
impl Component for User {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::Entry(_) => {},
        }
    }
}
impl View for User {
    fn view(&self) -> Node<Self::Msg> {
        div![
            h1!["Profile"],
            p![self.name()],
            p![format!("Followers: {}", self.followers().len())],
        ]
    }
}
#[async_trait(?Send)]
impl TableItem for User {
    fn table_route() -> route::Route {
        route::Route::Users
    }
    fn entry_route(id: Id<Self>) -> route::Route {
        route::Route::User(id)
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        api::get_users()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        api::get_user(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        api::delete_user(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}

impl Child<entry::Model<Self>> for User {
    fn parent_msg(msg: Self::Msg) -> Option<entry::Msg<Self>> {
        match msg {
            Msg::Entry(msg) => Some(*msg),
        }
    }
}

impl Preview for User {
    fn preview(&self) -> Node<Msg> {
        div![
            p!["Preview"],
            a![
                attrs!{
                    At::Href => "";
                },
                self.name(),
                simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Preview(Box::new(preview::Msg::Open))))),
            ],
            self.followers().len(),
        ]
    }
}
