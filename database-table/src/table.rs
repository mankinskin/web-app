use rql::*;
use crate::{
    entry::*,
    route::*,
};
use async_trait::async_trait;
use std::fmt::Debug;
use enum_paths::{
    AsPath,
};
use serde::{
    Serialize,
    Deserialize,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};

use seed::{
    browser::fetch::{
        fetch as seed_fetch,
        Request,
        Method,
    },
};
use std::result::Result;
pub trait Routed : TableRoutable + Sized {
    type AbsoluteRoute: Route;
    fn to_absolute_route(route: <Self as TableRoutable>::Route) -> Self::AbsoluteRoute;
}
#[async_trait(?Send)]
pub trait RemoteTable<T=Self>
    : TableRoutable<T>
    + From<T>
    + Clone
    + Debug
{
    type Error: Debug + Clone;
    async fn get(id: Id<T>) -> Result<Option<Entry<T>>, Self::Error>;
    async fn delete(id: Id<T>) -> Result<Option<T>, Self::Error>;
    async fn get_all() -> Result<Vec<Entry<T>>, Self::Error>;
    //async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String>;
    async fn post(data: T) -> Result<Id<T>, Self::Error>;
}
async fn fetch<V>(request: Request<'_>) -> Result<V, String>
    where V: 'static + for<'de> Deserialize<'de>,
{
    seed_fetch(request).await
        .map_err(|e| format!("Fetch error: {:?}", e))?
        .json().await
        .map_err(|e| format!("Value error: {:?}", e))
}
#[async_trait(?Send)]
impl<T> RemoteTable for T
    where T: Debug
          + Clone
          + TableRoutable
          + for<'de> Deserialize<'de>
          + Serialize
          + Routed
{
    type Error = String;
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, Self::Error> {
        let path = Self::to_absolute_route(Self::entry_route(id)).as_path();
        debug!("RemoteTable::get {}", path);
        fetch(
            Request::new(path)
                .method(Method::Get)
        ).await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, Self::Error> {
        let path = Self::to_absolute_route(Self::entry_route(id)).as_path();
        debug!("RemoteTable::delete {}", path);
        fetch(
            Request::new(path)
                .method(Method::Delete)
        ).await
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, Self::Error> {
        let path = Self::to_absolute_route(Self::table_route()).as_path();
        debug!("RemoteTable::get_all {}", path);
        fetch(
            Request::new(path)
                .method(Method::Get)
        ).await
    }
    async fn post(data: Self) -> Result<Id<Self>, Self::Error> {
        let path = Self::to_absolute_route(Self::table_route()).as_path();
        debug!("RemoteTable::post {}", path);
        fetch(
            Request::new(path)
                .method(Method::Post)
                .json(&data)
                .map_err(|e| format!("{:?}", e))?
        ).await
    }
}
// todo when specialization is stable
//#[async_trait(?Send)]
//impl<T: RemoteTable<T>, U> RemoteTable<T> for U {
//  type Error = <T as RemoteTable>::Error;
//  async fn get(id: Id<T>) -> Result<Option<Entry<T>>, Self::Error> {
//      T::get(id).await
//  }
//  async fn delete(id: Id<T>) -> Result<Option<T>, Self::Error> {
//      T::delete(id).await
//  }
//  async fn get_all() -> Result<Vec<Entry<T>>, Self::Error> {
//      T::get_all().await
//  }
//  async fn post(data: T) -> Result<Id<T>, Self::Error> {
//      T::post(data).await
//  }
//}
pub trait DatabaseTable<'db, D>
    : Sized
    + Clone
    + Serialize
    + for<'de> Deserialize<'de>
    + 'db
    where D: crate::Database<'db, Self>,
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
impl<'db, T, D> DatabaseTable<'db, D> for T
    where D: crate::Database<'db, T>,
          T: Sized
          + Clone
          + Serialize
          + for<'de> Deserialize<'de>
          + 'db
{}
