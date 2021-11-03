use crate::{
    preview,
    Component,
    Init,
    Viewable,
};
use database_table::{
    Entry,
    RemoteTable,
};
use seed::{
    prelude::*,
    *,
};
use std::fmt::Debug;
use std::default::Default;
use std::result::Result;

#[derive(Debug, Clone, Default)]
pub struct List<D: RemoteTable, T: Component + RemoteTable<D> = D> {
    items: Vec<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<D: RemoteTable, T: Component + RemoteTable<D>> List<D, T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            _ty: Default::default(),
        }
    }
}
impl<D: RemoteTable, T: Component + RemoteTable<D>> Init<Msg<D, T>> for List<D, T> {
    fn init(msg: Msg<D, T>, orders: &mut impl Orders<Msg<D, T>>) -> Self {
        orders.send_msg(msg);
        Self::new()
    }
}
impl<D: RemoteTable, T: Component + RemoteTable<D>> From<Vec<Entry<D>>> for List<D, T> {
    fn from(entries: Vec<Entry<D>>) -> Self {
        Self {
            items: into_items(entries),
            _ty: Default::default(),
        }
    }
}
fn into_items<D: RemoteTable, T: Component + RemoteTable<D>>(entries: Vec<Entry<D>>) -> Vec<T> {
    entries.into_iter().map(|e| e.into_inner().into()).collect()
}
#[derive(Debug, Clone)]
pub enum Msg<D: RemoteTable, T: Component + RemoteTable<D> = D> {
    GetAll,
    All(Result<Vec<Entry<D>>, <T as RemoteTable<D>>::Error>),
    Item(usize, <T as Component>::Msg),
}
impl<D: RemoteTable, T: Component + RemoteTable<D>> Component for List<D, T> {
    type Msg = Msg<D, T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Msg<D, T>>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    T::get_all().map(Msg::<D, T>::All)
                    );
            }
            Msg::All(res) => {
                match res {
                    Ok(entries) => self.items = into_items(entries),
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Item(index, msg) => {
                //if let preview::Msg::Entry(entry::Msg::Deleted(_)) = msg {
                //	self.previews.remove(index);
                //} else {
                self.items[index].update(
                    msg,
                    &mut orders.proxy(move |msg| Msg::Item(index.clone(), msg)),
                    );
                //}
            }
        }
    }
}
impl<D: RemoteTable, T: Component + preview::Previewable + RemoteTable<D>> Viewable for List<D, T> {
    fn view(&self) -> Node<Msg<D, T>> {
        div![ul![self.items.iter().enumerate().map(|(i, item)| {
            li![item.preview()
                .map_msg(move |msg| Msg::Item(i.clone(), msg))]
        })]]
    }
}
