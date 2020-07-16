use seed::{
    self,
    Url,
};
use crate::{
    routes::{
        Route,
        TableRoutable,
    },
};
use plans::{
    user::User,
    project::Project,
    task::Task,
};
use rql::{
    Id,
};
use updatable::{
    *,
};
use database::{
    *,
};
use crate::{
    *,
};
use futures::future::FutureExt;
use async_trait::async_trait;

impl From<Route> for Url {
    fn from(route: Route) -> Self {
        Self::new().set_path(<Route as Into<Vec<String>>>::into(route))
    }
}
impl From<Url> for Route {
    fn from(url: Url) -> Self {
        Self::from(url.path())
    }
}
#[async_trait(?Send)]
pub trait TableItem
    : Clone
    + 'static
    + Updatable
    + TableRoutable
{
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String>;
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String>;
    async fn get_all() -> Result<Vec<Entry<Self>>, String>;
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String>;
    async fn post(data: Self) -> Result<Id<Self>, String>;
}
#[async_trait(?Send)]
impl TableItem for Project {
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        get_projects()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        get_project(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        delete_project(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String> {
        update_project(id, update)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        post_project(data)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}
#[async_trait(?Send)]
impl TableItem for Task {
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        get_tasks()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        get_task(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        delete_task(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String> {
        update_task(id, update)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        post_task(data)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}
#[async_trait(?Send)]
impl TableItem for User {
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        get_users()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        get_user(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        delete_user(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String> {
        update_user(id, update)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        post_user(data)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}
