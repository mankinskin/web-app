use crate::{
    entry,
    Component,
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

pub trait Previewable: Component {
    fn preview(&self) -> Node<Self::Msg>;
}
impl<T: Component + RemoteTable + Previewable + Debug> Previewable for Entry<T> {
    fn preview(&self) -> Node<Self::Msg> {
        div![
            attrs! {
                At::Href => &self.id.route().as_path();
            },
            self.data.preview().map_msg(entry::Msg::Data)
        ]
    }
}
