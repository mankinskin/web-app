#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused)]
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate seed;
extern crate serde_json;
extern crate serde;
#[macro_use] extern crate define_api;

api! {
    fn add(a: u32, b: u32) -> u32 {
        a + b
    }
    fn concat(a: String, b: String) -> String {
        format!("{}{}", a, b)
    }
}

