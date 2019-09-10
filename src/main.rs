extern crate chrono;
extern crate tabular;
extern crate daggy;
extern crate nom;
extern crate azul;

mod currency;
mod file;
mod budget;
mod transaction;
mod purpose;
mod actor;
mod interpreter;
mod query;
#[macro_use] mod cartesian;

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
use azul::prelude::*;
use azul::text_layout::*;

fn main() {
    let mut budget = Budget::create("My Budget", Euro(140));
    budget.get(Euro(19)).set_partner("Papa".into());
    budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
    budget.give(Euro(49)).set_purpose("Fahrstunde".into());
    budget.give(Euro(19)).set_purpose("Essen".into()).set_partner("Papa".into());
    println!("{}", budget);
    let mut app = App::new(budget.clone(), AppConfig::default()).unwrap();

    let window = app.create_window(WindowCreateOptions::default(), css::native()).unwrap();
    app.run(window).unwrap();

    //interpreter::run().unwrap();
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
