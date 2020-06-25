#![feature(proc_macro_hygiene, decl_macro, concat_idents)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate uuid;
extern crate serde_json;
extern crate serde;
extern crate jsonwebtoken;
extern crate api;

use rocket::{
    response::{
        *,
    },
    http::{
        *,
    },
};
use std::{
    path::{
        Path,
    },
};
use std::io::{
    Result,
};
pub fn get_file<P: AsRef<Path>>(path: P) -> Result<NamedFile> {
    NamedFile::open(path)
}
static CLIENT_DIR: &'static str = "seed_client";

#[get("/<app>")]
fn get_html(app: &RawStr) -> Result<NamedFile> {
    let _ = app;
    get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/")]
fn get_root_html() -> Result<NamedFile> {
    get_html("".into())
}
#[get("/pkg/<file_name>")]
fn get_pkg_js(file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./{}/pkg/{}", CLIENT_DIR, file_name))
}

fn main() {
    rocket::ignite()
        .mount("/",
            routes![
                get_html,
                get_root_html,
                get_pkg_js,
                api::routes::add,
                api::routes::concat,
            ],
        )
        .launch();
}
