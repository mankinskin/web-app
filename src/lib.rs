#![recursion_limit="1000"]
extern crate chrono;
extern crate tabular;
extern crate daggy;
extern crate nom;
extern crate yew;
extern crate stdweb;
extern crate web_sys;
extern crate wasm_bindgen;
#[macro_use] extern crate lazy_static;

mod currency;
mod file;
mod budget;
mod transaction;
mod purpose;
mod person;
mod interpreter;
mod query;
#[macro_use] mod cartesian;
mod frontend;

use crate::currency::{Euro};
use crate::budget::{Budget};
// TODO
// - Serialize budget to .bud file
// - Deserialize budget from .bud file
// - test budget file reading and writing
// x implement transaction queries
//  x with single partner
//  x with single purpose
//  x with any of multiple partners
//  x with any of multiple purposes
//  x with all of multiple purposes
//  x find all within timespan
//  x find after time
//  x find before time
//  x find all with min amount
//  x find all with max amount
//  x find all expenses
//  x find all earnings
// - purpose relations
//  x A implies B
//  x use acyclic graph (with transitivity)
// -

use stdweb::{
    *,
    unstable::{
        TryInto,
    },
    web::{
        *,
        html_element::*,
    },
};

use web_sys::*;
use wasm_bindgen::prelude::*;
use yew::*;
use chrono::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    //let mut budget = Budget::create("My Budget", Euro(140));
    //budget.get(Euro(19)).set_partner("Papa".into());
    //budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
    //budget.give(Euro(49)).set_purpose("Fahrstunde".into());
    //budget.give(Euro(19)).set_purpose("Essen".into()).set_partner("Papa".into());
    //println!("{}", budget);

    //interpreter::run().unwrap();
    //console!(log, "yew::initialize");
    yew::initialize();

    //let document = web_sys::window().unwrap().document().unwrap();
    //let head = document.head().unwrap();
    //head.set_inner_text("<head>
    //  <link rel=\"stylesheet\" href=\"node_modules/xterm/css/xterm.css\" />
    //  <script src=\"node_modules/xterm/lib/xterm.js\"></script>
    //  <style>
    //      body {
    //          background-color: rgb(20, 20, 20);
    //      }
    //      h1, caption, table, td {
    //          font-family: Impact, Charcoal, sans-serif;
    //          color: lightblue;
    //      }
    //      table, th, td {
    //          border-bottom: 1px solid lightgray;
    //          border-collapse: collapse;
    //          padding: 10px;
    //      }
    //      .transaction {
    //          color: blue;
    //      }
    //      .flex-container {
    //          display: flex;
    //          flex-direction: row;
    //      }
    //      caption {
    //          font-weight: bold;
    //      }
    //  </style>
    //</head>");

    //console!(log, "App::new");
    yew::App::<Budget<Euro>>::new()
        .mount_to_body()
        .send_message(frontend::Msg::Init);
    //console!(log, "run_loop");
    yew::run_loop();
    Ok(())
}

mod tests {
    #[test]
    fn new_budget_file() {
        use crate::file::{
            create_budget_file,
            delete_budget_file,
        };
        let _budget_file = create_budget_file("test").unwrap();
        delete_budget_file("test").unwrap()
    }
    #[test]
    fn open_budget_file() {

    }
    #[test]
    fn write_budget_file() {

    }
    mod transactions {
        #[test]
        fn give() {
            // write a giving transaction to budget file

        }
    }
}
