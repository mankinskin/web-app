use yew::{
    *,
};
pub trait Child : Sized + Component {
}

pub trait Parent<C> : Sized + Component
    where C: Child
{
    //fn children(&self) -> Vec<Self>;
    fn child_callback(link: &ComponentLink<Self>, child_index: usize) -> Callback<<C as Component>::Message>;
}

pub trait ChildMessage<C>
    where C: Child
{
    fn child_message(child_index: usize, msg: <C as Component>::Message) -> Self;
}
pub trait ChildProps<C>
    where C: Child + Sized + Component
{
    fn set_parent_callback(&mut self, callback: Callback<<C as Component>::Message>);
    fn get_parent_callback(&self) -> Callback<<C as Component>::Message>;
    fn update(&mut self, msg: <C as Component>::Message);
}
