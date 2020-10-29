use crate::{
    entry,
    Component,
    Viewable,
};
use database_table::{
    Entry,
    Routable,
    RemoteTable,
};
use enum_paths::AsPath;
use seed::{
    prelude::*,
    *,
};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Model<T: TableItem + Debug> {
    pub entry: Entry<T>,
}
impl<T: Component + TableItem + Debug> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Self {
        Model { entry }
    }
}
#[derive(Clone, Debug)]
pub enum Msg<T: Component + TableItem + Debug> {
    Entry(entry::Msg<T>),
}
impl<T: Component + TableItem + Debug> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry
                    .update(msg.clone(), &mut orders.proxy(Msg::Entry));
            }
        }
    }
}
impl<D, T: Component + RemoteTable<D> + Debug + Previewable> Viewable for Preview<D, T> {
    fn view(&self) -> Node<Self::Msg> {
        self.entry.preview().map_msg(Msg::Entry)
    }
}

pub trait Previewable: Component {
    fn preview(&self) -> Node<Self::Msg>;
}
impl<D, T: Component + RemoteTable<D> + Previewable + Debug> Previewable for Entry<T> {
    fn preview(&self) -> Node<Self::Msg> {
        div![
            attrs! {
                At::Href => &self.id.route().as_path();
            },
            self.data.preview().map_msg(entry::Msg::Data)
        ]
    }
}
