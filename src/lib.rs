#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate derive_builder;
#[cfg(not(target_arch = "wasm32"))]
extern crate database_table;
extern crate enum_paths;
#[cfg(not(target_arch = "wasm32"))]
extern crate http;
#[cfg(not(target_arch = "wasm32"))]
extern crate jsonwebtoken;
extern crate lazy_static;
#[cfg(not(target_arch = "wasm32"))]
extern crate rql;
#[cfg(not(target_arch = "wasm32"))]
extern crate updatable;

pub mod auth;
pub mod project;
pub mod route;
pub mod task;
pub mod user;

use lazy_static::lazy_static;
use rql::*;
schema! {
    pub Schema {
        user: user::User,
        task: task::Task,
        project: project::Project,
    }
}

lazy_static! {
    pub static ref DB: Schema = Schema::new("test_database", rql::HumanReadable).unwrap();
}
