use crate::{
    project::*,
    task::*,
    user::*,
};
use database_table::{
    Routable,
};
use rql::*;

use enum_paths::{
    AsPath,
    ParseError,
    ParsePath,
};

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Chart,
    #[name = ""]
    Auth(AuthRoute),
    Users,
    #[name = ""]
    User(Id<User>),
    Projects,
    #[name = ""]
    Project(Id<Project>),
    Task(Id<Task>),
    #[name = ""]
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
