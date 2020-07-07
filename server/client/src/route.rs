use seed::{
    self,
};
use rql::{
    *,
};
use plans::{
    user::*,
    project::*,
    task::*,
};
use crate::{
    page,
    entry::{
        self,
        Model,
        TableItem,
    },
    config::{
        Child,
    },
};
use std::str::{
    FromStr,
};
#[derive(Clone, Debug)]
pub enum Route {
    Home,
    Login,
    Register,
    Users,
    User(Id<User>),
    Projects,
    Project(Id<Project>),
    Task(Id<Task>),
    NotFound,
}
impl Into<Vec<String>> for Route {
    fn into(self) -> Vec<String> {
        match self {
            Route::NotFound => vec![],
            Route::Home => vec![],
            Route::Login => vec!["login".into()],
            Route::Register => vec!["register".into()],
            Route::Users => vec!["user".into()],
            Route::User(id) => vec!["user".into(), id.to_string()],
            Route::Projects => vec!["project".into()],
            Route::Project(id) => vec!["project".into(), id.to_string()],
            Route::Task(id) => vec!["task".into(), id.to_string()],
        }
    }
}
impl From<&[String]> for Route {
    fn from(path: &[String]) -> Self {
        if path.is_empty() {
            Route::Home
        } else {
            match &path[0][..] {
                "login" => Route::Login,
                "register" => Route::Register,
                "user" =>
                    if path.len() == 1 {
                        Route::Users
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::User(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                "project" =>
                    if path.len() == 1 {
                        Route::Projects
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::Project(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                "task" =>
                    if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::Task(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                _ => Route::Home,
            }
        }
    }
}
impl From<Route> for seed::Url {
    fn from(route: Route) -> Self {
        Self::new().set_path(<Route as Into<Vec<String>>>::into(route))
    }
}
impl From<seed::Url> for Route {
    fn from(url: seed::Url) -> Self {
        Self::from(url.path())
    }
}
impl From<page::Model> for Route {
    fn from(config: page::Model) -> Self {
        match config {
            page::Model::Home(_) | page::Model::NotFound => Route::Home,
            page::Model::Login(_) => Route::Login,
            page::Model::Register(_) => Route::Register,
            page::Model::UserList(_) => Route::Users,
            page::Model::UserProfile(profile) => Route::User(profile.entry.id),
            page::Model::ProjectProfile(profile) => Route::Project(profile.entry.id),
            page::Model::ProjectList(_) => Route::Projects,
            page::Model::TaskProfile(profile) => Route::Task(profile.entry.id),
        }
    }
}
pub trait Routable {
    fn route(&self) -> Route;
}

impl Routable for Route
{
    fn route(&self) -> Route {
        self.clone()
    }
}
impl<T: TableItem> Routable for T
{
    fn route(&self) -> Route {
        T::table_route()
    }
}
impl<T: TableItem> Routable for Id<T>
{
    fn route(&self) -> Route {
        T::entry_route(*self)
    }
}
impl<T: TableItem + Child<Model<T>>> Routable for entry::Model<T>
    where Id<T>: Routable,
{
    fn route(&self) -> Route {
        self.id.route()
    }
}
