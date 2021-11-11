#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]

extern crate test;

mod graph;
mod r#match;
mod merge;
mod read;
mod search;
mod split;
mod vertex;

#[cfg(test)]
pub use graph::tests::*;
pub use graph::*;
pub(crate) use merge::*;
pub(crate) use read::*;
pub use search::*;
pub use split::*;
pub use vertex::*;
