extern crate seed;
extern crate app_model;

pub mod login;
pub mod register;

use seed::{
    prelude::*,
};

use std::fmt::Debug;
pub trait ComponentMsg
    : Clone + Debug + 'static
{}
impl<T: Clone + Debug + 'static> ComponentMsg for T
{}
pub trait Component{
    type Msg: ComponentMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
}
pub trait View : Component {
    fn view(&self) -> Node<Self::Msg>;
}
pub trait Init<T> : Component
    where
        T: Clone,
{
    fn init(config: T, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self;
}
impl<T, C> Init<T> for C
    where
        C: Component,
        T: Into<C> + Into<<C as Component>::Msg> + Clone
{
    fn init(config: T, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        orders.send_msg(config.clone().into());
        config.into()
    }
}
