use crate::{
    entry::{
        TableItem,
    },
    entry,
    preview,
};
use seed::{
    prelude::*,
};
use database::{
    Entry,
};

pub trait Component {
    type Msg: Clone + 'static;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
}
pub trait Child<P: Component> : Component {
    fn parent_msg(msg: Self::Msg) -> Option<P::Msg>;
}
impl<T: TableItem> Child<preview::Model<T>> for Entry<T> {
    fn parent_msg(msg: Self::Msg) -> Option<preview::Msg<T>> {
        match msg {
            entry::Msg::Preview(msg) => Some(*msg),
            _ => None
        }
    }
}
pub trait View : Component {
    fn view(&self) -> Node<Self::Msg>;
}
impl<T: TableItem + View> View for Entry<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.data.view().map_msg(entry::Msg::Data)
    }
}
pub trait Config<C> : Clone
    where
        C: Component,
{
    fn into_model(self, orders: &mut impl Orders<<C as Component>::Msg>) -> C;
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg>);
    fn init(self, orders: &mut impl Orders<<C as Component>::Msg>) -> C {
        self.clone().send_msg(orders);
        self.into_model(orders)
    }
}
impl<C, T> Config<C> for T
    where
        C: Component,
        T: Into<C> + Into<<C as Component>::Msg> + Clone
{
    fn send_msg(self, orders: &mut impl Orders<<C as Component>::Msg>) {
        orders.send_msg(self.into());
    }
    fn into_model(self, _orders: &mut impl Orders<<C as Component>::Msg>) -> C {
        self.into()
    }
}
