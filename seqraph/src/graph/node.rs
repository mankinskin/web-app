use std::fmt::{
    Debug,
    Display,
};

pub trait NodeData: Debug + PartialEq + Clone {}
impl<T: Debug + PartialEq + Clone> NodeData for T {}
