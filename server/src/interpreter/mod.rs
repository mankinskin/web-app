#![allow(unused)]
mod error;
pub mod parse;

use crate::currency::*;
use crate::transaction::*;
use parse::*;
use ::chrono::*;
use crate::person::*;

use std::io::{
    self,
    Read,
    Write,
};
fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn print_help() {
    println!("\
    Natural language interpreter

    q[uit] | exit | :q\tQuit interpreter.
    h[elp] | ?\t\tShow help.
    ");
}
pub fn run() -> io::Result<()> {
    println!("Running interpreter ");
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        trim_newline(&mut input); // remove newline
        match &input as &str {
            "q" | "quit" | "exit" | ":q" => break,
            "h" | "help" | "?" => print_help(),
            _ => println!("{:?}", Transaction::<Euro>::parse(&input)),
        };
    }
    Ok(())
}
