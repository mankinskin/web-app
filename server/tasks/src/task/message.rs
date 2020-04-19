use super::{
    TaskNodeView,
};
use common::{
    expander::{self, *},
    parent_child::*,
};
use yew::{
    *,
};
#[derive(Clone, Debug)]
pub enum Msg {
    ChildMessage(usize, Box<Msg>),
    NodeMessage(NodeMsg),
    TaskMessage(TaskMsg),
    Focussed,
}
#[derive(Clone, Debug)]
pub enum NodeMsg {
    ExpanderMessage(Box<expander::Msg<TaskNodeView>>),
    Focussed,
}
#[derive(Clone, Debug)]
pub enum TaskMsg {
    UpdateDescription(String),
}
impl ChildMessage<ExpanderView<TaskNodeView>> for Msg {
    fn child_message(child_index: usize, msg: <ExpanderView<TaskNodeView> as Component>::Message) -> Self {
        Msg::ChildMessage(child_index, Box::new(Msg::NodeMessage(NodeMsg::ExpanderMessage(Box::new(msg)))))
    }
}
impl ChildMessage<TaskNodeView> for Msg {
    fn child_message(child_index: usize, msg: <TaskNodeView as Component>::Message) -> Self {
        Msg::ChildMessage(child_index, Box::new(msg))
    }
}
