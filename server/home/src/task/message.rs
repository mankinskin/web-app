use super::{
    TaskNodeView,
    TaskView,
};
use components::{
    expander::{self, ExpanderView},
    parent_child::*,
};
use yew::{
    *,
};
#[derive(Clone, Debug)]
pub enum RootMsg {
    Passthrough(TaskMsg),
    Focussed(FocusEvent),
}
#[derive(Clone, Debug)]
pub enum TaskMsg {
    UpdateDescription(String),
    ChildMessage(usize, Box<NodeMsg>),
}
#[derive(Clone, Debug)]
pub enum NodeMsg {
    ExpanderMessage(Box<expander::Msg>),
    Focussed(FocusEvent),
    Passthrough(TaskMsg),
}
impl ToChildMessage<TaskView> for TaskMsg {
    fn to_child_message(&self, child_index: usize) -> ChildMessage<TaskView> {
        ChildMessage::message(child_index, self.clone())
    }
}
impl FromChildMessage<TaskView> for RootMsg {
    fn from_child_message(msg: ChildMessage<TaskView>) -> Self {
        RootMsg::Passthrough(msg.message.clone())
    }
}
impl FromChildMessage<TaskView> for NodeMsg {
    fn from_child_message(msg: ChildMessage<TaskView>) -> Self {
        NodeMsg::Passthrough(msg.message.clone())
    }
}
impl ToChildMessage<TaskNodeView> for NodeMsg {
    fn to_child_message(&self, child_index: usize) -> ChildMessage<TaskNodeView> {
        ChildMessage::message(child_index, self.clone())
    }
}
impl FromChildMessage<TaskNodeView> for TaskMsg {
    fn from_child_message(msg: ChildMessage<TaskNodeView>) -> Self {
        TaskMsg::ChildMessage(msg.index().clone(), Box::new(msg.message.clone()))
    }
}
impl FromChildMessage<ExpanderView> for NodeMsg {
    fn from_child_message(message: ChildMessage<ExpanderView>) -> Self {
        NodeMsg::ExpanderMessage(Box::new(message.message))
    }
}
