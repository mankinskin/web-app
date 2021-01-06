use crate::{
    editor::Edit,
    Component,
    Viewable,
};
use database_table::RemoteTable;
use rql::Id;
use seed::prelude::*;
use std::fmt::Debug;
#[allow(unused)]
use tracing::{
    debug,
    info,
};

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
#[derive(Debug, Clone)]
pub enum Msg<T: Component + RemoteTable> {
    Post,
    Posted(Id<T>),
    Data(<T as Component>::Msg),
}
impl<T: Component + RemoteTable + Debug + Clone> Component for NewData<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Post => {
                debug!("editor::Msg::Post");
                let data = self.data.clone();
                orders.perform_cmd(async move {
                    debug!("posting...");
                    T::post(data).await
                        .map(Msg::Posted)
                        .expect("Failed to post data")
                });
            }
            Msg::Posted(id) => {
                debug!("{:?}", id);
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
