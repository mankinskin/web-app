use super::{
    Msg,
    ExpanderView,
};
use crate::{
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};

#[derive(Properties, Clone, Debug)]
pub struct ExpanderData<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    pub element: <C as Component>::Properties,
    pub expanded: bool,
    pub message_parent: Callback<<ExpanderView<C> as Component>::Message>,
}
impl<C> ChildProps<ExpanderView<C>> for ExpanderData<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    fn set_parent_callback(&mut self, callback: Callback<<ExpanderView<C> as Component>::Message>) {
        self.message_parent = callback;
    }
    fn get_parent_callback(&self)-> Callback<<ExpanderView<C> as Component>::Message> {
        self.message_parent.clone()
    }
    fn update(&mut self, msg: Msg<C>) {
        match msg.clone() {
            Msg::ToggleExpand => {
                console!(log, format!("Toggle"));
                self.expanded = !self.expanded;
            },
            Msg::ChildMessage(msg) => {
                console!(log, format!("Child Message"));
                self.element.update(msg)
            },
        }
    }
}
