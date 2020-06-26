#![feature(async_closure)]
extern crate lazy_static;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate anyhow;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate rql;
extern crate plans;
extern crate database;
extern crate updatable;
extern crate seed;
extern crate api;

pub mod login;
pub mod register;
pub mod home;
pub mod navbar;
pub mod users;
pub mod page;
pub mod root;
pub use root::*;
pub mod route;
pub mod projects;
pub mod tasks;
