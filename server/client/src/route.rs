use seed::{
    self,
};
use rql::{
    *,
};
use plans::{
    user::*,
    //project::*,
    //task::*,
};
use crate::{
    page,
};
use std::str::{
    FromStr,
};
#[derive(Clone, Debug)]
pub enum Route {
    Home,
    Login,
    Register,
    UserList,
    UserProfile(Id<User>),
    //Projects,
    //Project(Id<Project>),
    //Task(Id<Task>),
    NotFound,
}
impl Into<Vec<String>> for Route {
    fn into(self) -> Vec<String> {
        match self {
            Route::NotFound => vec![],
            Route::Home => vec![],
            Route::Login => vec!["login".into()],
            Route::Register => vec!["register".into()],
            Route::UserList => vec!["users".into()],
            Route::UserProfile(id) => vec!["users".into(), id.to_string()],
            //Route::Projects => vec!["projects".into()],
            //Route::Project(id) => vec!["projects".into(), id.to_string()],
            //Route::Task(id) => vec!["tasks".into(), id.to_string()],
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
                        Route::UserList
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::UserProfile(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                //"projects" =>
                //    if path.len() == 1 {
                //        Route::Projects
                //    } else if path.len() == 2 {
                //        match Id::from_str(&path[1]) {
                //            Ok(id) => Route::Project(id),
                //            Err(_e) => Route::NotFound,
                //        }
                //    } else {
                //        Route::NotFound
                //    },
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
            page::Model::UserList(_) => Route::UserList,
            page::Model::UserProfile(profile) => Route::UserProfile(profile.user_id.clone()),
            //page::Model::Project(project) => Route::Project(project.project_id),
            //page::Model::Projects(_) => Route::Projects,
            //page::Model::Task(task) => Route::Task(task.task_id),
        }
    }
}
