use crate::{
    Component,
    Viewable,
};
use database_table::{
    Entry,
    RemoteTable,
};
use seed::{
    prelude::*,
};
use std::fmt::Debug;

impl<T: RemoteTable + Viewable> Viewable for Entry<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.data.view().map_msg(Msg::Data)
    }
}
#[derive(Debug, Clone)]
pub enum Msg<T: RemoteTable + Component + Debug> {
    Refresh,
    Refreshed(Result<Option<Entry<T>>, <T as RemoteTable>::Error>),

    Delete,
    Deleted(Result<Option<T>, <T as RemoteTable>::Error>),

    Update,
    Updated(Result<Option<T>, <T as RemoteTable>::Error>),

    Data(<T as Component>::Msg),
}
use futures::future::FutureExt;
impl<T: RemoteTable + Component> Component for Entry<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Refresh => {
                orders.perform_cmd(
                    T::get(self.id).map(Msg::Refreshed)
                );
            }
            Msg::Refreshed(res) => {
                match res {
                    Ok(r) => {
                        if let Some(entry) = r {
                            *self = entry;
                        }
                    }
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Delete => {
                orders.perform_cmd(
                    T::delete(self.id).map(Msg::Deleted)
                );
            }
            Msg::Deleted(res) => {
                match res {
                    Ok(r) => {
                        seed::log(r);
                    }
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Update => {
                //orders.perform_cmd(
                //    <T as RemoteTable>::update(self.id, T::Update::from(self.data.clone()))
                //        .map(|res| Msg::Updated(res))
                //);
            }
            Msg::Updated(res) => {
                match res {
                    Ok(r) => {
                        seed::log(r);
                    }
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Data(msg) => {
                self.data.update(msg, &mut orders.proxy(Msg::Data));
            }
        }
    }
}
