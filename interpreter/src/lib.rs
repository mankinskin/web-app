#![allow(unused)]

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
