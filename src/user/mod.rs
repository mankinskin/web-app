use crate::{
    auth::credentials::*,
};
#[cfg(target_arch = "wasm32")]
use components::{
    entry,
    preview,
    Component,
    Viewable,
};
use database_table::{
    TableItem,
    TableRoutable,
};
use rql::*;
#[cfg(target_arch = "wasm32")]
use seed::{
    prelude::*,
    *,
};
use serde::{
    Serialize,
    Deserialize,
};
use std::fmt::{
    self,
    Display,
};
use enum_paths::{
    AsPath,
};
//#[cfg(target_arch = "wasm32")]
//pub mod profile;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    credentials: Credentials,
    full_name: Option<String>,
    followers: Vec<Id<User>>,
}
impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self {
            credentials,
            full_name: None,
            followers: vec![],
        }
    }
}
impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.credentials.username)
    }
}
impl User {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn new<S1: ToString, S2: ToString>(name: S1, password: S2) -> Self {
        Self {
            credentials: Credentials::new(name, password),
            full_name: None,
            followers: vec![],
        }
    }
    pub fn name(&self) -> &String {
        &self.credentials.username
    }
    pub fn password(&self) -> &String {
        &self.credentials.password
    }
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
    pub fn credentials_mut(&mut self) -> &mut Credentials {
        &mut self.credentials
    }
    pub fn followers(&self) -> &Vec<Id<User>> {
        &self.followers
    }
    pub fn full_name(&self) -> &Option<String> {
        &self.full_name
    }
}
#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Users,
    #[as_path = ""]
    User(Id<User>),
}
impl database_table::Route for Route {}
impl TableRoutable for User {
    type Route = Route;
    fn table_route() -> Route {
        Route::Users
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::User(id)
    }
}
impl TableItem for User {}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub enum Msg {
    Entry(Box<entry::Msg<User>>),
}
#[cfg(target_arch = "wasm32")]
impl Component for User {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Entry(_) => {}
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Viewable for User {
    fn view(&self) -> Node<Self::Msg> {
        div![
            h1!["Profile"],
            p![self.name()],
            p![format!("Followers: {}", self.followers().len())],
        ]
    }
}
#[cfg(target_arch = "wasm32")]
impl preview::Preview for User {
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
