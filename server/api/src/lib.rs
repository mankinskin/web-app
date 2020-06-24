#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate seed;
extern crate serde_json;
extern crate serde;

use serde::*;
use seed::{
    *,
    prelude::*,
    browser::fetch::{
        Method,
        FetchError,
    },
};
use rocket::{
    request::{
        FromParam,
    },
    response::{
        *,
    },
    http::{
        *,
    },
};
use std::result::{
    Result,
};
use rocket_contrib::{
    json::*,
};
//#[api]
//fn add(a: u32, b: u32) -> u32 {
//    a + b
//}
// becomes:

#[derive(Deserialize, Serialize)]
pub struct addParameters {
    a: u32,
    b: u32,
}
#[derive(Deserialize, Serialize)]
pub struct addResult(u32);

// server side (rocket)
#[cfg(not(target_arch="wasm32"))]
pub mod call {
    pub(crate) fn add(a: u32, b: u32) -> u32 {
        a + b
    }
}
#[cfg(not(target_arch="wasm32"))]
pub mod routes {
    use super::*;
    #[post("/api/call/add", data="<parameters>")]
    pub fn add(parameters: Json<addParameters>) -> Json<addResult> {
        let Json(parameters) = parameters;
        Json(addResult(call::add(parameters.a, parameters.b)))
    }
}


// client side (seed)
#[cfg(target_arch="wasm32")]
pub async fn add(a: u32, b: u32) -> Result<u32, FetchError> {
    let route = "/api/call/add";
    let url = format!("{}{}", "http://localhost:8000", route);
    // authentication
    //if let Some(session) = root::get_session() {
    //    req = req.header(Header::authorization(format!("{}", session.token)));
    //}
    seed::fetch::fetch(
        seed::fetch::Request::new(&url)
            .method(Method::Post)
            .json(&addParameters { a, b })?
    )
    .await?
    .check_status()?
    .json()
    .await
    .map(|res: addResult| res.0)
}
