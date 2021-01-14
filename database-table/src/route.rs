use rql::*;
use enum_paths::{
    AsPath,
    ParsePath,
};
use crate::entry::*;
use std::fmt::Debug;

pub trait Route : Clone + Debug + AsPath + ParsePath + 'static{ }
pub trait Routable {
    type Route: Route;
    fn route(&self) -> Self::Route;
}

impl<T: TableRoutable> Routable for T {
    type Route = T::Route;
    fn route(&self) -> Self::Route {
        T::table_route()
    }
}
impl<T: TableRoutable> Routable for Entry<T> {
    type Route = T::Route;
    fn route(&self) -> Self::Route {
        T::entry_route(self.id)
    }
}
impl<T: TableRoutable> Routable for Id<T> {
    type Route = T::Route;
    fn route(&self) -> Self::Route {
        T::entry_route(*self)
    }
}
pub trait TableRoutable<T=Self>: 'static {
    type Route: Route;
    fn table_route() -> Self::Route;
    fn entry_route(id: Id<T>) -> Self::Route;
}
