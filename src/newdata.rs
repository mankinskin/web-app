use crate::{
    editor::Edit,
    Component,
    Viewable,
};
use database_table::RemoteTable;
use rql::Id;
use seed::prelude::*;
use std::fmt::Debug;
use std::result::Result;

#[derive(Debug, Clone)]
pub struct NewData<T> {
    pub data: T,
}
impl<T: Default> Default for NewData<T> {
    fn default() -> Self {
        Self::from(T::default())
    }
}
impl<T> From<T> for NewData<T> {
    fn from(data: T) -> Self {
        Self { data }
    }
}
use futures::future::FutureExt;
#[derive(Debug)]
pub enum Msg<T: Component + RemoteTable> {
    Post,
    Posted(Result<Id<T>, <T as RemoteTable>::Error>),
    Data(<T as Component>::Msg),
}
impl<T: Component + RemoteTable + Debug + Clone> Component for NewData<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Post => {
                orders.perform_cmd(
                    T::post(self.data.clone()).map(Msg::Posted)
                );
            }
            Msg::Posted(res) => {
                match res {
                    Ok(id) => {
                        seed::log(id);
                    }
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Data(msg) => self.data.update(msg, &mut orders.proxy(Msg::Data)),
        }
    }
}
impl<T: Viewable + RemoteTable + Debug + Clone> Viewable for NewData<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.data.view().map_msg(Msg::Data)
    }
}
impl<T: Edit + RemoteTable + Debug + Clone> Edit for NewData<T> {
    fn edit(&self) -> Node<Self::Msg> {
        self.data.edit().map_msg(Msg::Data)
    }
}
