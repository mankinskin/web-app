use crate::{
    root,
};
use seed::{
    prelude::*,
};

pub trait Component {
    type Msg;
}
pub trait Config<C> : Clone
    where
        C: Component,
        <C as Component>::Msg: 'static,
{
    fn into_model(self, orders: &mut impl Orders<<C as Component>::Msg, root::GMsg>) -> C;
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg, root::GMsg>);
    fn init(self, orders: &mut impl Orders<<C as Component>::Msg, root::GMsg>) -> C {
        self.clone().send_msg(orders);
        self.into_model(orders)
    }
}
impl<C, T> Config<C> for T
    where
        C: Component,
        <C as Component>::Msg: 'static,
        T: Into<C> + Into<<C as Component>::Msg> + Clone
{
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg, root::GMsg>) {
        orders.send_msg(self.into());
    }
    fn into_model(self, _orders: &mut impl Orders<<C as Component>::Msg, root::GMsg>) -> C {
        self.into()
    }
}
