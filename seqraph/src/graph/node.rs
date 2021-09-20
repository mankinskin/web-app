use std::fmt::Debug;

pub trait NodeData: Debug + PartialEq + Clone {}
impl<T: Debug + PartialEq + Clone> NodeData for T {}
