extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate http;
extern crate anyhow;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate rql;
extern crate plans;
extern crate budget;
extern crate updatable;
extern crate database;

use seed::{
    *,
    prelude::*,
};
use rql::{
    *,
};
use plans::{
    user::*,
};
pub mod login;
pub mod register;
pub mod home;
pub mod navbar;
pub mod user;
pub mod page;
pub mod root;

use root::{
    Msg,
};
fn routes(url: Url) -> Option<Msg> {
    let path = url.path.join("/");
    match &path[..] {
        "" => Some(Msg::SetPage(page::Model::home())),
        "login" => Some(Msg::SetPage(page::Model::login())),
        "register" => Some(Msg::SetPage(page::Model::register())),
        "users" => Some(Msg::SetPage(page::Model::home())),
        _ => Some(Msg::SetPage(page::Model::home())),
    }
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(root::update, root::view)
        .routes(routes)
        .build_and_start();
}
