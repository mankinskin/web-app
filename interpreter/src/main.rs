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
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod shell;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use shell::*;

//pub mod text;

//use text::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn main() {
    let mut shell = Shell::new();
    shell.set_prompt("shell> ");
    shell.run().unwrap()
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() { }
