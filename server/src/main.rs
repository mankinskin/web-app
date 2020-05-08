#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate rql;
extern crate uuid;
extern crate chrono;
extern crate colored;
extern crate serde_json;
extern crate serde;
extern crate plans;
#[macro_use] extern crate anyhow;

mod server;
mod database;

fn main() {
    database::setup();
    server::start()
}
