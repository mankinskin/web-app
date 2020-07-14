use plans::{
    user::*,
};
use crate::{
    config::{
        Component,
        View,
    },
    entry::{
        self,
    },
    preview::{
        *,
    },
};
use database::{
    Entry,
};
use seed::{
    *,
    prelude::*,
};

pub mod profile;

#[derive(Clone)]
pub enum Msg {
    Entry(Box<entry::Msg<User>>),
}
impl Component for User {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
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
impl Preview for User {
    fn preview(&self) -> Node<Msg> {
        div![
            p!["Preview"],
            a![
                self.name(),
                //ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Preview(Box::new(preview::Msg::Open))))),
            ],
            self.followers().len(),
        ]
    }
}
