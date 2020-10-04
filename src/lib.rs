extern crate lazy_static;
extern crate seed;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_wasm;
extern crate database_table;
extern crate enum_paths;
extern crate rql;

pub mod entry;
pub mod preview;
pub mod editor;
pub use editor::{
    Edit,
    Editor,
};
pub mod remote;
pub mod newdata;
pub mod list;

use seed::prelude::*;

use std::fmt::Debug;
pub trait Init<Cfg>: Component {
    fn init(config: Cfg, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self;
}
impl<Cfg, Cmp> Init<Cfg> for Cmp
where
    Self: Component,
    Cfg: Into<Cmp>,
{
    fn init(config: Cfg, _orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        config.into()
    }
}
pub trait ComponentMsg: Clone + Debug + 'static {}
impl<T: Clone + Debug + 'static> ComponentMsg for T {}
pub trait Component {
    type Msg: ComponentMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
}
pub trait Viewable: Component {
    fn view(&self) -> Node<Self::Msg>;
}
