use std::fmt::{
	Debug,
	Display,
};

pub trait NodeData: Debug + Display + PartialEq + Clone {}
impl<T: Debug + Display + PartialEq + Clone> NodeData for T {}
