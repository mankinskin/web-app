#![allow(unused)]
#![feature(split_inclusive)]

extern crate nom;
#[macro_use] extern crate itertools;
extern crate nom_unicode;
extern crate chrono;
extern crate petgraph;
extern crate pretty_assertions;
#[macro_use] extern crate lazy_static;
extern crate nalgebra;
extern crate num_traits;
extern crate serde;
extern crate seqraph;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate linefeed;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate termion;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;

pub mod parse;
//pub mod text;
//pub mod sentence;
//pub mod graph;
//pub mod set;
