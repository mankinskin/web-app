#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused)]

#[cfg(target_arch = "wasm32")]
mod client;
#[cfg(target_arch = "wasm32")]
pub use client::*;

#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;

use app_model::{
    project::Project,
    task::Task,
    user::User,
};
use database_table::*;
use futures::future::FutureExt;
use interpreter::*;
use rql::*;
use seqraph::*;
use updatable::*;
use define_api::api;



api! {
    fn get_project_tasks(id: Id<Project>) -> Vec<Entry<Task>> {
        let ids = <Project as DatabaseTable<'_, Schema>>::get(id)
            .map(|entry| entry.data().tasks().clone())
            .unwrap_or(Vec::new());
        <Task as DatabaseTable<'_, Schema>>::get_list(ids)
    }
    fn get_user_projects(id: Id<User>) -> Vec<Entry<Project>> {
        <Project as DatabaseTable<'_, Schema>>::filter(|project| project.members().contains(&id))
    }
    //fn project_create_subtask(project: Id<Project>, task: Task) -> Id<Task> {
    //    let id = <Task as DatabaseTable>::insert(task);
    //    <Project as DatabaseTable>::update(project, Project::update().tasks(vec![id.clone()]));
    //    id
    //}
    fn interpret_text(text: String) -> String {
        let mut g = TG.lock().unwrap();
        g.read_sequence(text.chars());
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
