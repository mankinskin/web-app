#![recursion_limit="1000"]
extern crate chrono;
extern crate tabular;
extern crate daggy;
extern crate nom;
extern crate yew;
extern crate stdweb;

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
// - implement transaction queries
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
use yew::*;
use chrono::*;

fn main() {
    //let mut budget = Budget::create("My Budget", Euro(140));
    //budget.get(Euro(19)).set_partner("Papa".into());
    //budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
    //budget.give(Euro(49)).set_purpose("Fahrstunde".into());
    //budget.give(Euro(19)).set_purpose("Essen".into()).set_partner("Papa".into());
    //println!("{}", budget);

    //interpreter::run().unwrap();
    console!(log, "yew::initialize");
    yew::initialize();

    let body =
        document()
        .body()
        .expect("Failed to get document body element");
    let mount_class = "mount-point";
    let mount_point = document().create_element("div").unwrap();
    mount_point.class_list().add(mount_class).unwrap();
    body.append_child(&mount_point);

    console!(log, "App::new");
    yew::App::<Budget<Euro>>::new()
        .mount_as_body()
        .send_message(frontend::Msg::Init);
    console!(log, "run_loop");
    yew::run_loop();
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
