use crate::{
    entry,
};
use api::{
    TableItem,
};
use seed::{
    prelude::*,
};
use database::{
    Entry,
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
impl<T: TableItem + View + Debug> View for Entry<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.data.view().map_msg(entry::Msg::Data)
    }
}
pub trait Config<C> : Clone
    where
        C: Component,
{
    fn init(self, orders: &mut impl Orders<<C as Component>::Msg>) -> C;
}
impl<C, T> Config<C> for T
    where
        C: Component,
        T: Into<C> + Into<<C as Component>::Msg> + Clone
{
    fn init(self, orders: &mut impl Orders<<C as Component>::Msg>) -> C {
        orders.send_msg(self.clone().into());
        self.into()
    }
}
