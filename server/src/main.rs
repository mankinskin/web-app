#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rouille;
extern crate rql;
extern crate chrono;
extern crate colored;
extern crate serde_json;
extern crate plans;

mod server;
mod database;

fn main() {
    database::setup();
    server::start()
}
