#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused)]
#[macro_use] extern crate define_api;
extern crate updatable;
extern crate database_table;
extern crate interpreter;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate seed;
extern crate serde_json;
extern crate serde;
extern crate rql;
extern crate async_trait;
extern crate futures;
extern crate seqraph;
extern crate app_model;

#[cfg(target_arch="wasm32")]
extern crate components;

#[cfg(target_arch="wasm32")]
mod client;
#[cfg(target_arch="wasm32")]
pub use client::*;
#[cfg(not(target_arch="wasm32"))]
mod server;
#[cfg(not(target_arch="wasm32"))]
pub use server::*;

use app_model::{
    Route,
    Project,
    Task,
    User,
};
use rql::{
    Id,
};
use updatable::{
    *,
};
use database_table::{
    *,
};
use interpreter::{
    *,
};
use seqraph::{
    *,
};
use futures::future::FutureExt;
use std::sync::Mutex;

#[cfg(not(target_arch="wasm32"))]
lazy_static! {
    static ref TG: Mutex<SequenceGraph<char>> = Mutex::new(SequenceGraph::new());
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
    fn interpret_text(text: String) -> String {
        let mut g = TG.lock().unwrap();
        g.learn_sequence(text.chars());
        g.write_to_file("graphs/g1").unwrap();
        "Done".into()
    }
    fn query_text(query: String) -> Option<NodeInfo<char>> {
        TG.lock().unwrap().query(query.chars())
    }
    rest_api!(User);
    rest_api!(Project);
    rest_api!(Task);
}
