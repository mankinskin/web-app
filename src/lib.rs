extern crate app_model;
extern crate seed;
extern crate lazy_static;

pub mod auth;

use seed::prelude::*;

use std::fmt::Debug;
pub trait Init<Cfg: Clone>: Component {
    fn init(config: Cfg, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self;
}
impl<Cfg, Cmp> Init<Cfg> for Cmp
where
    Self: Component,
    Cfg: Into<Cmp> + Into<<Self as Component>::Msg> + Clone,
{
    fn init(config: Cfg, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        orders.send_msg(config.clone().into());
        config.into()
    }
}
pub trait ComponentMsg: Clone + Debug + 'static
{}
impl<T: Clone + Debug + 'static> ComponentMsg for T
{}
pub trait Component {
    type Msg: ComponentMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
}
pub trait Viewable: Component {
    fn view(&self) -> Node<Self::Msg>;
}
