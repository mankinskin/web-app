#![allow(unused)]

extern crate nom;
#[macro_use] extern crate itertools;
extern crate nom_unicode;
extern crate chrono;
extern crate petgraph;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate linefeed;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate termion;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;

pub mod parse;
pub mod text;
pub mod graph;
