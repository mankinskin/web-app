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
    users,
    tasks,
    projects,
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
    UserProfile(Id<User>),
    Projects,
    Project(Id<Project>),
    Task(Id<Task>),
    NotFound,
}
impl Into<Vec<String>> for Route {
    fn into(self) -> Vec<String> {
        match self {
            Route::Home => vec![],
            Route::Login => vec!["login".into()],
            Route::Register => vec!["register".into()],
            Route::Users => vec!["users".into()],
            Route::UserProfile(id) => vec!["users".into(), id.to_string()],
            Route::Projects => vec!["projects".into()],
            Route::Project(id) => vec!["projects".into(), id.to_string()],
            Route::Task(id) => vec!["tasks".into(), id.to_string()],
            Route::NotFound => vec![],
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
                "users" =>
                    if path.len() == 1 {
                        Route::Users
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::UserProfile(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                "projects" =>
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
impl From<page::Config> for Route {
    fn from(config: page::Config) -> Self {
        match config {
            page::Config::UserProfile(config) => Route::UserProfile(
                match config {
                    users::profile::Config::UserId(id) => id,
                    users::profile::Config::Model(model) => model.user_id,
                }
            ),
            page::Config::Users(_) => Route::Users,
            page::Config::Project(config) => Route::Project(
                match config {
                    projects::project::Config::ProjectId(id) => id,
                    projects::project::Config::Entry(entry) => *entry.id(),
                }
            ),
            page::Config::Projects(_) => Route::Projects,
            page::Config::Task(config) => Route::Task(
                match config {
                    tasks::task::Config::TaskId(id) => id,
                    tasks::task::Config::Model(model) => model.task_id,
                    tasks::task::Config::Entry(entry) => *entry.id(),
                }
            ),
            page::Config::Login(_) => Route::Login,
            page::Config::Register(_) => Route::Register,
            page::Config::Home(_) => Route::Home,
        }
    }
}
impl From<page::Model> for Route {
    fn from(config: page::Model) -> Self {
        match config {
            page::Model::UserProfile(profile) => Route::UserProfile(profile.user.user_id),
            page::Model::Users(_) => Route::Users,
            page::Model::Project(project) => Route::Project(project.project_id),
            page::Model::Projects(_) => Route::Projects,
            page::Model::Task(task) => Route::Task(task.task_id),
            page::Model::Login(_) => Route::Login,
            page::Model::Register(_) => Route::Register,
            page::Model::Home(_) | page::Model::NotFound => Route::Home,
        }
    }
}
