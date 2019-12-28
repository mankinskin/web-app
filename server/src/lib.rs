#![recursion_limit="1000"]
extern crate chrono;
extern crate tabular;
extern crate daggy;
extern crate nom;

pub mod currency;
pub mod file;
pub mod budget;
pub mod interpreter;
pub mod transaction;
pub mod purpose;
pub mod person;
pub mod query;
#[macro_use] pub mod cartesian;

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
