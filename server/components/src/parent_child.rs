use yew::{
    *,
};

pub trait Focussable<E> {
    fn focus(&self) -> Callback<E>;
}
pub trait MessageUpdate<M> {
    fn update(&mut self, msg: M);
}
pub trait ChildProps<C> : Properties
    where C: Component<Properties=Self>,
        <C as Component>::Message: ToChildMessage<C>,
{
    fn set_parent_callback(&mut self, callback: Callback<<C as Component>::Message>);
}

/// Parents can create callbacks for their children to receive messages
/// The callback returns the Component::Message for the parent
pub trait IParent<C> : Component
    where C: Component,
        <Self as Component>::Message: FromChildMessage<C>,
        <C as Component>::Message: ToChildMessage<C>,
{
    fn link(&self) -> &ComponentLink<Self>;
    fn child_callback(&self, child_index: usize) -> Callback<<C as Component>::Message> {
        self.link().callback(move |msg: <C as Component>::Message| {
            let child_message: ChildMessage<C> = msg.to_child_message(child_index);
            <Self as Component>::Message::from_child_message(child_message)
        })
    }
}
/// a message from a parent's child
pub struct ChildMessage<C>
    where C: Component,
{
    index: usize,
    pub message: <C as Component>::Message,
}
impl<C> ChildMessage<C>
    where C: Component,
        <C as Component>::Message: ToChildMessage<C>,
{

    pub fn index(&self) -> &usize {
        &self.index
    }
    pub fn message(index: usize, message: <C as Component>::Message) -> Self {
        Self {
            index,
            message,
        }
    }
}
/// A trait for messages convertible to ChildMessage<C>
pub trait ToChildMessage<C>
    where C: Component,
{
    fn to_child_message(&self, child_index: usize) -> ChildMessage<C>;
}
/// A trait for messages convertible from ChildMessage<C>
pub trait FromChildMessage<C>
    where C: Component,
        <C as Component>::Message: ToChildMessage<C>,
{
    fn from_child_message(message: ChildMessage<C>) -> Self;
}
