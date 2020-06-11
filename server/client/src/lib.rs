#![recursion_limit = "1024"]
extern crate yew;
extern crate yew_router;
#[macro_use] extern crate stdweb;
extern crate wasm_bindgen;
extern crate http;
extern crate anyhow;
extern crate serde_json;
extern crate serde;
extern crate rql;
extern crate url;
extern crate wasm_bindgen_futures;
extern crate futures;
extern crate js_sys;
extern crate web_sys;

extern crate plans;
extern crate budget;
extern crate components;
extern crate updatable;
extern crate database;

mod transaction;
mod transactions;
mod budget_view;
mod router;
mod root;
mod user_profile;
mod note;
mod login;
mod signup;
mod task;
mod tasks;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::initialize();
    yew::App::<root::ClientRoot>::new()
        .mount_to_body();
    yew::run_loop();
    Ok(())
}
