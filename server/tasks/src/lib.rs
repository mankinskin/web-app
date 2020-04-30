#![recursion_limit = "1024"]
extern crate yew;
extern crate yew_router;
#[macro_use] extern crate stdweb;
extern crate wasm_bindgen;
#[macro_use] extern crate anyhow;
extern crate serde_json;
extern crate serde;
extern crate rql;
extern crate url;
extern crate wasm_bindgen_futures;
extern crate futures;
extern crate js_sys;
extern crate web_sys;

extern crate plans;
extern crate common;

mod router;
mod task;
mod page;
mod remote_data;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::initialize();
    yew::App::<router::ClientRouter>::new()
        .mount_to_body();
    yew::run_loop();
    Ok(())
}
