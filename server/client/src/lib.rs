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
extern crate database_table;
extern crate interpreter;
extern crate updatable;
extern crate seed;
extern crate app_model;
extern crate async_trait;
extern crate console_error_panic_hook;
extern crate seqraph;
extern crate components;
extern crate enum_paths;

pub mod root;

pub mod page;
pub mod home;
pub mod navbar;

pub use root::*;
