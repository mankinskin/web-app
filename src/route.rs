use crate::{project::*, task::*, user::*};
use database_table::Entry;
use rql::*;
use std::str::FromStr;
use updatable::Updatable;
pub trait TableRoutable: Clone + 'static + Updatable {
    fn table_route() -> Route;
    fn entry_route(id: Id<Self>) -> Route;
}
impl TableRoutable for Project {
    fn table_route() -> Route {
        Route::Projects
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::Project(id)
    }
}
impl TableRoutable for Task {
    fn table_route() -> Route {
        Route::Home
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::Task(id)
    }
}
impl TableRoutable for User {
    fn table_route() -> Route {
        Route::Users
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::User(id)
    }
}
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
pub trait Routable {
    fn route(&self) -> Route;
}
impl Routable for Route {
    fn route(&self) -> Route {
        self.clone()
    }
}
impl<T: TableRoutable> Routable for T {
    fn route(&self) -> Route {
        T::table_route()
    }
}
impl<T: TableRoutable> Routable for Id<T> {
    fn route(&self) -> Route {
        T::entry_route(*self)
    }
}
impl<T: TableRoutable> Routable for Entry<T> {
    fn route(&self) -> Route {
        T::entry_route(self.id)
    }
}
impl Into<Vec<String>> for Route {
    fn into(self) -> Vec<String> {
        match self {
            Route::NotFound => vec![],
            Route::Home => vec![],
            Route::Login => vec!["login".into()],
            Route::Register => vec!["register".into()],
            Route::Users => vec!["users".into()],
            Route::User(id) => vec!["users".into(), id.to_string()],
            Route::Projects => vec!["projects".into()],
            Route::Project(id) => vec!["projects".into(), id.to_string()],
            Route::Task(id) => vec!["tasks".into(), id.to_string()],
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
                "users" => {
                    if path.len() == 1 {
                        Route::Users
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::User(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    }
                }
                "projects" => {
                    if path.len() == 1 {
                        Route::Projects
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::Project(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    }
                }
                "tasks" => {
                    if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::Task(id),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    }
                }
                _ => Route::Home,
            }
        }
    }
}
impl ToString for Route {
    fn to_string(&self) -> String {
        let v: Vec<String> = self.clone().into();
        v.iter()
            .fold(String::from("/"), |a, x| format!("{}{}/", a, x))
    }
}
