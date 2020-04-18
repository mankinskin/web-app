use crate::{
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};
#[derive(Clone, Debug)]
pub enum Msg<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    ToggleExpand,
    ChildMessage(<C as Component>::Message),
}
impl<C> ChildMessage<C> for Msg<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    fn child_message(_: usize, msg: <C as Component>::Message) -> Self {
        console!(log, format!("child message {:#?}", msg));
        Msg::ChildMessage(msg)
    }
}
