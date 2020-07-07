use crate::{
    root::GMsg,
};
use seed::{
    prelude::*,
};

pub trait Component {
    type Msg: Clone + 'static;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>);
}
pub trait Child<P: Component> : Component {
    fn parent_msg(msg: Self::Msg) -> Option<P::Msg>;
}
pub trait View : Component {
    fn view(&self) -> Node<Self::Msg>;
}
pub trait Config<C> : Clone
    where
        C: Component,
{
    fn into_model(self, orders: &mut impl Orders<<C as Component>::Msg, GMsg>) -> C;
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg, GMsg>);
    fn init(self, orders: &mut impl Orders<<C as Component>::Msg, GMsg>) -> C {
        self.clone().send_msg(orders);
        self.into_model(orders)
    }
}
impl<C, T> Config<C> for T
    where
        C: Component,
        T: Into<C> + Into<<C as Component>::Msg> + Clone
{
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg, GMsg>) {
        orders.send_msg(self.into());
    }
    fn into_model(self, _orders: &mut impl Orders<<C as Component>::Msg, GMsg>) -> C {
        self.into()
    }
}
