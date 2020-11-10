pub mod credentials;
use enum_paths::AsPath;
use rql::Id;
use serde::{
    Deserialize,
    Serialize,
};
use crate::user::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;

#[cfg(target_arch = "wasm32")]
pub mod client;
#[cfg(target_arch = "wasm32")]
pub use client::{
    *,
    Msg,
};

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Login,
    Register,
}
impl database_table::Route for Route {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Id<User>,
    pub token: String,
}
