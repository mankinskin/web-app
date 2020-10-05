#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate derive_builder;
extern crate components;
#[cfg(not(target_arch = "wasm32"))]
extern crate database_table;
extern crate enum_paths;
#[cfg(not(target_arch = "wasm32"))]
extern crate http;
#[cfg(not(target_arch = "wasm32"))]
extern crate jsonwebtoken;
extern crate lazy_static;
extern crate openlimits;
#[cfg(not(target_arch = "wasm32"))]
extern crate rocket;
#[cfg(not(target_arch = "wasm32"))]
extern crate rql;
#[cfg(target_arch = "wasm32")]
extern crate seed;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_wasm;
#[cfg(not(target_arch = "wasm32"))]
extern crate updatable;

pub mod auth;
pub use auth::*;
pub mod project;
pub use project::*;
pub mod route;
pub use route::*;
pub mod task;
pub use task::*;
pub mod user;
pub use user::*;
pub mod market;

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
    pub static ref DB: Schema = Schema::new("test_database", rql::BinaryStable).unwrap();
}
