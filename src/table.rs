use rql::*;
use crate::{
    entry::*,
};
use enum_paths::{
    AsPath,
    ParsePath,
};
use async_trait::async_trait;
use std::fmt::Debug;
use serde::{
    Serialize,
    Deserialize,
};

pub trait Routable {
    type Route: Route;
    fn route(&self) -> Self::Route;
}
pub trait Route : Clone + Debug + AsPath + ParsePath + 'static{ }
pub trait TableRoutable: Clone + 'static {
    type Route: Route;
    fn table_route() -> Self::Route;
    fn entry_route(id: Id<Self>) -> Self::Route;
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
//use std::result::Result;
#[async_trait(?Send)]
pub trait TableItem
    : Clone
    + 'static
    + TableRoutable
{
    //async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String>;
    //async fn delete(id: Id<Self>) -> Result<Option<Self>, String>;
    //async fn get_all() -> Result<Vec<Entry<Self>>, String>;
    //async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String>;
    //async fn post(data: Self) -> Result<Id<Self>, String>;
}
pub trait DatabaseTable<'db, D: crate::Database<'db, Self>>
    : Sized
    + Clone
    + Serialize
    + for<'de> Deserialize<'de>
    + 'db
{
    fn table() -> TableGuard<'db, Self> {
        D::table()
    }
    fn table_mut() -> TableGuardMut<'db, Self> {
        D::table_mut()
    }
    fn insert(obj: Self) -> Id<Self> {
        D::insert(obj)
    }
    fn get(id: Id<Self>) -> Option<Entry<Self>> {
        D::get(id)
    }
    fn delete(id: Id<Self>) -> Option<Self> {
        D::delete(id)
    }
    fn get_all() -> Vec<Entry<Self>> {
        D::get_all()
    }
    fn get_list(ids: Vec<Id<Self>>) -> Vec<Entry<Self>> {
        D::get_list(ids)
    }
    fn filter<F>(f: F) -> Vec<Entry<Self>>
        where F: Fn(&Self) -> bool
    {
        D::filter(f)
    }
    fn find<F>(f: F) -> Option<Entry<Self>>
        where F: Fn(&Self) -> bool
    {
        D::find(f)
    }
}
impl<'db, T, D: crate::Database<'db, T>> DatabaseTable<'db, D> for T
    where T: Sized
    + Clone
    + Serialize
    + for<'de> Deserialize<'de>
    + 'db
{}
