use seed::{
    *,
    prelude::*,
};
use rql::{
    Id,
};
use updatable::{
    Updatable,
};
use crate::{
    config::{
        Component,
        Child,
    },
    preview::{self},
    route::{
        self,
    },
};
use database::{
    Entry,
};
use std::result::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait TableItem: Clone + 'static + Child<Entry<Self>> + Updatable {
    fn table_route() -> route::Route;
    fn entry_route(id: Id<Self>) -> route::Route;
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String>;
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String>;
    async fn get_all() -> Result<Vec<Entry<Self>>, String>;
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String>;
    async fn post(data: Self) -> Result<Id<Self>, String>;
}
#[derive(Clone)]
pub enum Msg<T: TableItem + Component> {
    Refresh,
    Refreshed(Result<Option<Entry<T>>, String>),

    Delete,
    Deleted(Result<Option<T>, String>),

    Update,
    Updated(Result<Option<T>, String>),

    Data(<T as Component>::Msg),
    Preview(Box<preview::Msg<T>>),
}
impl<T: TableItem + Component> Component for Entry<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Refresh => {
                orders.perform_cmd(
                    T::get(self.id).map(|res| Msg::Refreshed(res))
                );
            },
            Msg::Refreshed(res) => {
                match res {
                    Ok(r) =>
                        if let Some(entry) = r {
                            *self = entry;
                        },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Delete => {
                orders.perform_cmd(
                    T::delete(self.id).map(|res| Msg::Deleted(res))
                );
            },
            Msg::Deleted(res) => {
                match res {
                    Ok(r) => { seed::log(r); },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Update => {
                orders.perform_cmd(
                    <T as TableItem>::update(self.id, T::Update::from(self.data.clone()))
                        .map(|res| Msg::Updated(res))
                );
            },
            Msg::Updated(res) => {
                match res {
                    Ok(r) => { seed::log(r); },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Data(msg) => {
                self.data.update(msg.clone(), &mut orders.proxy(Msg::Data));
                T::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            Msg::Preview(_) => {
            },
        }
    }
}
