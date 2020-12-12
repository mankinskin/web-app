use crate::{
    editor::Edit,
    entry,
    preview::{
        Previewable,
    },
    Component,
    Init,
    Viewable,
};
use database_table::{
    Entry,
    Routable,
    RemoteTable,
};
use rql::Id;
use seed::{
    prelude::*,
    *,
};
use std::fmt::Debug;
use std::result::Result;
use futures::future::FutureExt;

#[derive(Debug, Clone)]
pub enum Remote<T: RemoteTable> {
    Loading(Id<T>),
    Ready(Entry<T>),
}
impl<T: RemoteTable + Component + Debug> Init<Id<T>> for Remote<T> {
    fn init(id: Id<T>, orders: &mut impl Orders<Msg<T>>) -> Self {
        orders.send_msg(Msg::Get);
        Self::Loading(id)
    }
}
impl<T: RemoteTable + Component> From<Entry<T>> for Remote<T> {
    fn from(entry: Entry<T>) -> Self {
        Self::Ready(entry)
    }
}
#[derive(Debug, Clone)]
pub enum Msg<T: Component + RemoteTable> {
    Get,
    Got(Result<Option<Entry<T>>, <T as RemoteTable>::Error>),
    Entry(entry::Msg<T>),
}
impl<T: RemoteTable + Component + Debug> Component for Remote<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match self {
            Self::Loading(id) => {
                match msg {
                    Msg::Get => {
                        orders.perform_cmd(
                            T::get(*id).map(Msg::Got)
                        );
                    }
                    Msg::Got(res) => {
                        match res {
                            Ok(r) => {
                                if let Some(entry) = r {
                                    *self = Self::Ready(entry);
                                }
                            }
                            Err(e) => {
                                seed::log(e);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Self::Ready(entry) => {
                match msg {
                    Msg::Entry(msg) => {
                        entry.update(msg, &mut orders.proxy(Msg::Entry));
                    }
                    Msg::Get => {
                        entry.update(entry::Msg::Refresh, &mut orders.proxy(Msg::Entry));
                    }
                    _ => {}
                }
            }
        }
    }
}
impl<T: RemoteTable + Previewable + Debug> Previewable for Remote<T> {
    fn preview(&self) -> Node<Self::Msg> {
        match self {
            Self::Ready(entry) => entry.preview().map_msg(Msg::Entry),
            Self::Loading(_) => div![h1!["Preview"], p!["Loading..."],],
        }
    }
}
impl<T: RemoteTable + Viewable + Debug> Viewable for Remote<T> {
    fn view(&self) -> Node<Self::Msg> {
        match self {
            Self::Ready(entry) => entry.view().map_msg(Msg::Entry),
            Self::Loading(_) => div![h1!["Viewable"], p!["Loading..."],],
        }
    }
}
impl<T: RemoteTable + Edit + Debug> Edit for Remote<T> {
    fn edit(&self) -> Node<Self::Msg> {
        match self {
            Self::Ready(entry) => entry.edit().map_msg(Msg::Entry),
            Self::Loading(_) => div![h1!["Editor"], p!["Loading..."],],
        }
    }
}
impl<T: RemoteTable> Routable for Remote<T> {
    type Route = T::Route;
    fn route(&self) -> Self::Route {
        match self {
            Self::Ready(entry) => entry.route(),
            Self::Loading(id) => id.route(),
        }
    }
}
