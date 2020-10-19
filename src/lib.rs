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
    pub static ref DB: Schema = Schema::new("app_model_database", rql::BinaryStable).unwrap();
}
