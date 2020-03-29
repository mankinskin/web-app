#![recursion_limit = "1024"]
extern crate yew;
extern crate yew_router;
#[macro_use] extern crate stdweb;
extern crate wasm_bindgen;
#[macro_use] extern crate lazy_static;
extern crate plans;

mod transaction;
mod transactions;
mod budget;
mod router;

use wasm_bindgen::prelude::*;
use plans::{
    currency::*,
};

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::initialize();
    yew::App::<router::ClientRouter>::new()
        .mount_to_body();
    yew::run_loop();
    Ok(())
}