use std::fmt::Debug;

pub trait EdgeData: Debug + PartialEq + Clone {}
impl<T: Debug + PartialEq + Clone> EdgeData for T {}
