use seed::{
    *,
    prelude::*,
};
use rql::{
    Id,
};
use crate::{
    config::{
        Config,
        Component,
        View,
    },
    preview::{
        self,
        Preview,
    },
    editor::{
        Edit,
    },
    entry::{
        self,
    },
};
use api::{
    routes::{
        Route,
        Routable,
    },
    TableItem,
};
use database::{
    Entry,
};
use std::result::Result;
use std::fmt::Debug;

#[derive(Debug,Clone)]
pub enum Model<T: TableItem> {
    Loading(Id<T>),
    Ready(Entry<T>),
}
impl<T: TableItem + Component + Debug> Config<Model<T>> for Id<T> {
    fn into_model(self, _orders: &mut impl Orders<Msg<T>>) -> Model<T> {
        Model::Loading(self)
    }
    fn send_msg(self, orders: &mut impl Orders<Msg<T>>) {
        orders.send_msg(Msg::Get);
    }
}
impl<T: TableItem + Component> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Model<T> {
        Model::Ready(entry)
    }
}
#[derive(Clone, Debug)]
pub enum Msg<T: TableItem + Component + Debug> {
    Get,
    Got(Result<Option<Entry<T>>, String>),
    Entry(entry::Msg<T>),
    Preview(Box<preview::Msg<T>>),
}
impl<T: TableItem + Component + Debug> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match self {
            Model::Loading(id) => {
                match msg {
                    Msg::Get => {
                        orders.perform_cmd(
                            T::get(*id).map(|res| Msg::Got(res))
                        );
                    },
                    Msg::Got(res) => {
                        match res {
                            Ok(r) =>
                                if let Some(entry) = r {
                                    *self = Model::Ready(entry);
                                },
                            Err(e) => { seed::log(e); },
                        }
                    },
                    _ => {},
                }
            },
            Model::Ready(entry) => {
                match msg {
                    Msg::Entry(msg) => {
                        entry.update(
                            msg.clone(),
                            &mut orders.proxy(Msg::Entry)
                        );
                    },
                    Msg::Preview(_) => {
                    },
                    Msg::Get => {
                        entry.update(
                            entry::Msg::Refresh,
                            &mut orders.proxy(Msg::Entry)
                        );
                    },
                    _ => {},
                }
            },
        }
    }
}
impl<T: TableItem + Preview + Debug> Preview for Model<T> {
    fn preview(&self) -> Node<Self::Msg> {
        match self {
            Model::Ready(entry) => {
                entry.preview().map_msg(Msg::Entry)
            },
            Model::Loading(_) => {
                div![
                    h1!["Preview"],
                    p!["Loading..."],
                ]
            },
        }
    }
}
impl<T: TableItem + View + Debug> View for Model<T> {
    fn view(&self) -> Node<Self::Msg> {
        match self {
            Model::Ready(entry) => {
                entry.view().map_msg(Msg::Entry)
            },
            Model::Loading(_) => {
                div![
                    h1!["View"],
                    p!["Loading..."],
                ]
            },
        }
    }
}
impl<T: TableItem + Edit + Debug> Edit for Model<T> {
    fn edit(&self) -> Node<Self::Msg> {
        match self {
            Model::Ready(entry) => {
                entry.edit().map_msg(Msg::Entry)
            },
            Model::Loading(_) => {
                div![
                    h1!["Editor"],
                    p!["Loading..."],
                ]
            },
        }
    }
}
impl<T: TableItem> Routable for Model<T>
    where Id<T>: Routable,
{
    fn route(&self) -> Route {
        match self {
            Model::Ready(entry) => entry.route(),
            Model::Loading(id) => id.route(),
        }
    }
}
