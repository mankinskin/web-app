#![allow(unused)]

extern crate nom;
#[macro_use] extern crate itertools;
extern crate nom_unicode;
extern crate chrono;
extern crate petgraph;
extern crate pretty_assertions;
#[macro_use] extern crate lazy_static;
extern crate nalgebra;
extern crate num_traits;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate linefeed;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate termion;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;

pub mod parse;
pub mod text;
pub mod sentence;
pub mod graph;

use std::collections::{HashSet};

#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
                temp_set // Return the populated HashSet
        }
    };
}
