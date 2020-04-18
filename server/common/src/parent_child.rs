use yew::{
    *,
};
pub trait Child : Sized + Component {
}

pub trait Parent<C> : Sized + Component
    where C: Child
{
    fn child_callback(&self, child_index: usize) -> Callback<<C as Component>::Message>;
    fn set_child_callbacks(&mut self);
}

pub trait ChildMessage<C>
    where C: Child
{
    fn child_message(child_index: usize, msg: <C as Component>::Message) -> Self;
}
pub trait UpdateProp<C>
    where C: Component + Sized
{
    fn update(&mut self, msg: <C as Component>::Message);
}
pub trait ChildProps<C>
    where C: Child + Sized + Component
{
    fn set_parent_callback(&mut self, callback: Callback<<C as Component>::Message>);
    fn get_parent_callback(&self) -> Callback<<C as Component>::Message>;
}
