#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused)]
#[macro_use] extern crate define_api;
extern crate updatable;
#[macro_use] extern crate lazy_static;

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate seed;
extern crate serde_json;
extern crate serde;

#[cfg(not(target_arch="wasm32"))]
extern crate jwt;

extern crate rql;
extern crate plans;
extern crate database;
extern crate async_trait;
extern crate futures;

pub mod routes;
use routes::{
    Route,
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
use futures::future::FutureExt;
use async_trait::async_trait;

pub trait TableRoutable
    : Clone
    + 'static
    + Updatable
{
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
#[cfg(target_arch="wasm32")]
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
#[cfg(target_arch="wasm32")]
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
impl TableRoutable for Task {
    fn table_route() -> Route {
        Route::Home
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::Task(id)
    }
}
#[cfg(target_arch="wasm32")]
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
impl TableRoutable for User {
    fn table_route() -> Route {
        Route::Users
    }
    fn entry_route(id: Id<Self>) -> Route {
        Route::User(id)
    }
}
#[cfg(target_arch="wasm32")]
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
api! {
    fn get_project_tasks(id: Id<Project>) -> Vec<Entry<Task>> {
        let ids = Project::get(id)
            .map(|entry| entry.data().tasks().clone())
            .unwrap_or(Vec::new());
        Task::get_list(ids)
    }
    fn get_user_projects(id: Id<User>) -> Vec<Entry<Project>> {
        Project::filter(|project| project.members().contains(&id))
    }
    fn project_create_subtask(project: Id<Project>, task: Task) -> Id<Task> {
        let id = <Task as DatabaseTable>::insert(task);
        <Project as DatabaseTable>::update(project, Project::update().tasks(vec![id.clone()]));
        id
    }
    rest_api!(User);
    rest_api!(Project);
    rest_api!(Task);
}
