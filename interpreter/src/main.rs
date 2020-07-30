extern crate seqraph;
#[macro_use] extern crate lazy_static;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use shell::*;
extern crate itertools;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn main() {
    let mut shell = Shell::new();
    shell.set_prompt("shell> ");
    shell.run().unwrap()
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() { }
