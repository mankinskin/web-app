#![feature(async_closure)]
extern crate anyhow;
extern crate app_model;
extern crate async_trait;
extern crate chrono;
extern crate components;
extern crate console_error_panic_hook;
extern crate database_table;
extern crate enum_paths;
extern crate futures;
extern crate interpreter;
extern crate lazy_static;
extern crate rql;
extern crate seed;
extern crate seqraph;
extern crate serde;
extern crate serde_json;
extern crate updatable;
extern crate url;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

pub mod root;

pub mod home;
pub mod navbar;
pub mod page;

pub use root::*;
