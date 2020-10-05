#![feature(clamp)]
extern crate app_model;
extern crate chrono;
extern crate components;
extern crate console_error_panic_hook;
extern crate enum_paths;
extern crate futures;
extern crate js_sys;
extern crate lazy_static;
extern crate openlimits;
extern crate rand;
extern crate rand_distr;
extern crate rust_decimal;
extern crate seed;
extern crate serde;
extern crate serde_json;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_wasm;
extern crate url;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

mod client;
mod shared;
pub use client::*;
