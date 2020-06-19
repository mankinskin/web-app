#[macro_use] extern crate lazy_static;
extern crate serde_json;
extern crate serde;
extern crate rql;
extern crate updatable;
extern crate plans;

pub mod entry;
pub use entry::*;
pub mod table;
pub use table::*;

pub fn setup() {
}
