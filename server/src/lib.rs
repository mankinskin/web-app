#![recursion_limit="1000"]
extern crate chrono;
extern crate tabular;
extern crate daggy;
extern crate nom;
#[macro_use] extern crate itertools;

pub mod currency;
pub mod budget;
pub mod interpreter;
pub mod transaction;
pub mod purpose;
pub mod subject;
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
    mod transactions {
        #[test]
        fn give() {
            // write a giving transaction to budget file

        }
    }
}
