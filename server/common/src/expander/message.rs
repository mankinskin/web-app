use crate::{
    parent_child::*,
    expander::*,
};
#[derive(Clone, Debug)]
pub enum Msg {
    ToggleExpand,
}
impl ToChildMessage<ExpanderView> for Msg {
    fn to_child_message(&self, child_index: usize) -> ChildMessage<ExpanderView> {
        ChildMessage::message(child_index, self.clone())
    }
}
