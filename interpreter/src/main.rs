extern crate regex;
extern crate regex_syntax;
extern crate seqraph;
pub mod parse;
extern crate itertools;
#[macro_use] extern crate lazy_static;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate chrono;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use shell::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn main() {
    let mut shell = Shell::new();
    shell.set_prompt("shell> ");
    shell.run().unwrap()
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() { }
