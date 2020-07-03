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

api! {
    use rql::{
        Id,
    };
    use updatable::{
        *,
    };
    use plans::{
        project::{
            Project,
        },
        user::{
            User,
        },
        task::{
            Task,
        },
    };
    use database::{
        *,
    };
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
