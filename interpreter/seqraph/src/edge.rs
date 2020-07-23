use std::fmt::{
    Debug,
};


pub trait EdgeData : Debug + PartialEq {}
impl<T: Debug + PartialEq> EdgeData for T {}

