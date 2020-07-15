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
extern crate async_trait;
extern crate console_error_panic_hook;

pub mod root;

pub mod config;
pub mod list;
pub mod preview;
pub mod entry;
pub mod newdata;
pub mod editor;
pub mod remote;

pub mod page;
pub mod login;
pub mod register;
pub mod home;
pub mod navbar;
pub mod user;
pub mod project;
pub mod task;

pub use root::*;
