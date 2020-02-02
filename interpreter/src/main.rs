// Components are things which can be combined to other components.
// components are part of a set of components.
// the component set defines the structure of the components value.
// an instance of a component can have exactly one value from the component set
// at any time. The component's set is also called the component's type.
//
// there can not be multiple instances of the exact same value, they would be
// exactly equal. This means they can not be distinguished by comparing them
// directly. To store multiple values of the same component type,
// you need to expand the type with distinguishable properties such as an
// index, a position or a name.
//
// Defining the set of Bit values:
//
// Bit := {0,1} = {0,1}
//
// An instance of a Bit is either


// required functions:
// - insert a new goal
// - define implications between states
// - given a state, compute the implied state
use interpreter::*;
use parse::*;

use std::io::{
    self,
    Write,
};

use interpreter::text::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use interpreter::shell::*;

fn print_help() {
    println!("Natural language interpreter

    q[uit] | exit | :q\tQuit interpreter.
    h[elp] | ?\t\tShow help.");
}
fn process_input(line: &str) -> io::Result<()> {
    let input = Text::parse(line).unwrap().1;
    println!("{:?}", input);
    Ok(())
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn run() -> io::Result<()> {
    println!("Running Budget App Interpreter");
    let mut shell = Shell::new();
    shell.set_prompt("shell> ");

    loop {
        print!("shell> ");
        std::io::stdout().flush().unwrap();
        //spawn(move || {
            //if let Some(key) = shell.keys().next() {
            //    match key.unwrap() {
            //        termion::event::Key::Up => println!("up"),
            //        _ => println!("something else"),
            //    }
            //}
            if let Some(line) = shell.lines().next() {
                let line = line.unwrap();
                match &line as &str {
                    "q" | "quit" | "exit" | ":q" => break,
                    "h" | "help" | "?" => print_help(),
                    //"history" => println!("{:?}", shell.get_history()),
                    line => process_input(line)?,
                }
                //shell.append_history_unique(line.clone());
            }
        //});
    }
    Ok(())
}
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn main() {
    run().unwrap();
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() {
}
