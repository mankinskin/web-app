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
    ExpanderMessage(usize, Box<expander::Msg<TaskNodeView>>),
    UpdateDescription(String),
    Noop,
}
impl ChildMessage<ExpanderView<TaskNodeView>> for Msg {
    fn child_message(child_index: usize, msg: <ExpanderView<TaskNodeView> as Component>::Message) -> Self {
        Msg::ExpanderMessage(child_index, Box::new(msg))
    }
}
