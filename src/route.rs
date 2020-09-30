use crate::{project::*, task::*, user::*};
use database_table::Entry;
use rql::*;
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
        Route::Root
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
use enum_paths::{AsPath, ParseError, ParsePath};

#[derive(Clone, AsPath)]
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
#[derive(Clone, AsPath)]
pub enum AuthRoute {
    Login,
    Register,
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
