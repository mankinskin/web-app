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

pub fn run() -> io::Result<()> {
    println!("Running interpreter ");
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.as_str() {
            "q" | "quit" | "exit" | ":q" => break,
            _ => println!("{:?}", Transaction::<Euro>::parse(&input)),
        };
    }
    Ok(())
}
