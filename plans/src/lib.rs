#[macro_use] extern crate serde;
extern crate serde_json;
#[macro_use] extern crate derive_builder;
#[macro_use] extern crate lazy_static;
extern crate jsonwebtoken;
extern crate rocket;

extern crate updatable;

pub mod task;
pub mod user;
pub mod note;
pub mod credentials;
pub mod jwt;
