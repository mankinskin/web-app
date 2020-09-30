#![feature(clamp)]
extern crate lazy_static;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate seed;
extern crate console_error_panic_hook;
extern crate components;
extern crate rand;
extern crate rand_distr;
extern crate openlimits;
extern crate rust_decimal;
extern crate web_sys;
extern crate js_sys;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_wasm;
extern crate enum_paths;

mod client;
pub use client::*;
mod shared;

