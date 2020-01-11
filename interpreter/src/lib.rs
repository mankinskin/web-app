#![allow(unused)]

extern crate nom;
#[macro_use] extern crate itertools;
//extern crate linefeed;
//extern crate termion;
extern crate chrono;

pub mod parse;
//pub mod shell;
pub mod text;

use parse::*;

use std::io::{
    self,
    Read,
    Write,
};

fn print_help() {
    println!("Natural language interpreter

    q[uit] | exit | :q\tQuit interpreter.
    h[elp] | ?\t\tShow help.");
}
//use shell::*;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{spawn};

pub fn run() -> io::Result<()> {
    println!("Running Budget App Interpreter");
    //let mut shell = Shell::new();
    //shell.set_prompt("shell> ");

    //loop {
    //    print!("shell> ");
    //    std::io::stdout().flush();
    //    //spawn(move || {
    //        //if let Some(key) = shell.keys().next() {
    //        //    match key.unwrap() {
    //        //        termion::event::Key::Up => println!("up"),
    //        //        _ => println!("something else"),
    //        //    }
    //        //}
    //        if let Some(line) = shell.lines().next() {
    //            let line = line.unwrap();
    //            match &line as &str {
    //                "q" | "quit" | "exit" | ":q" => break,
    //                "h" | "help" | "?" => print_help(),
    //                //"history" => println!("{:?}", shell.get_history()),
    //                line => {}//println!("{:?}", Transaction::<Euro>::parse(line)),
    //            }
    //            //shell.append_history_unique(line.clone());
    //        }
    //    //});
    //}
    Ok(())
}
