use super::{
    TaskView,
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
    ExpanderMessage(usize, Box<expander::Msg<TaskView>>),
    Noop,
}
impl ChildMessage<ExpanderView<TaskView>> for Msg {
    fn child_message(child_index: usize, msg: <ExpanderView<TaskView> as Component>::Message) -> Self {
        Msg::ExpanderMessage(child_index, Box::new(msg))
    }
}
