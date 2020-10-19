use crate::{
    project::*,
    task::*,
    user::*,
};
use database_table::Routable;
use rql::*;

use enum_paths::{
    AsPath,
    ParseError,
    ParsePath,
};

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Subscriptions,
    #[as_path = ""]
    Auth(AuthRoute),
    Users,
    #[as_path = ""]
    User(Id<User>),
    Projects,
    #[as_path = ""]
    Project(Id<Project>),
    Task(Id<Task>),
    #[as_path = ""]
    Root,
}
impl database_table::Route for Route {}
#[derive(Clone, Debug, AsPath)]
pub enum AuthRoute {
    Login,
    Register,
}
impl database_table::Route for AuthRoute {}

impl Routable for Route {
    type Route = Self;
    fn route(&self) -> Self::Route {
        self.clone()
    }
}
