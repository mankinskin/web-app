use crate::{
    entry,
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
pub struct List<D: Clone + Debug, T: RemoteTable<D> + Debug = D> {
    entries: Vec<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<D: Clone + Debug, T: Component + RemoteTable<D> + Debug> List<D, T> {
    pub fn new() -> Self {
        Self {
            previews: Vec::new(),
            _ty: Default::default(),
        }
    }
}
impl<D: Clone + Debug, T: Component + RemoteTable<D> + Default + Debug> Init<Msg<D, T>> for List<D, T>
{
    fn init(msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) -> List<T> {
        orders.send_msg(msg);
        List::default()
    }
}
impl<D: Clone + Debug, T: Component + RemoteTable<D> + Debug> From<Vec<Entry<T>>> for List<D, T>
{
    fn from(entries: Vec<Entry<T>>) -> Self {
        Self {
            previews: init_previews(entries),
            _ty: Default::default(),
        }
    }
}
fn init_previews<D: Clone + Debug, T: Component + RemoteTable<D> + Debug>(
    entries: Vec<Entry<D>>,
) -> Vec<preview::Preview<D, T>> {
    entries.into_iter().map(preview::Preview::from).collect()
}
#[derive(Debug)]
pub enum Msg<D: Clone + Debug, T: Component + RemoteTable<D> + Debug> {
    GetAll,
    All(Result<Vec<Entry<D>>, <T as RemoteTable<D>>::Error>),

    Preview(usize, preview::Msg<D, T>),
}
impl<D: Clone + Debug, T: Component + RemoteTable<D> + Debug> Component for List<D, T>
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    T::get_all().map(Msg::All)
                );
            }
            Msg::All(res) => {
                match res {
                    Ok(entries) => self.previews = init_previews(entries),
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Preview(index, msg) => {
                if let preview::Msg::Entry(entry::Msg::Deleted(_)) = msg {
                    self.previews.remove(index);
                } else {
                    self.previews[index].update(
                        msg,
                        &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg)),
                    );
                }
            }
        }
    }
}
impl<T: Component + preview::Previewable + RemoteTable + Debug> Viewable for List<D, T>
{
    fn view(&self) -> Node<Msg<T>> {
        div![ul![self.previews.iter().enumerate().map(|(i, preview)| {
            li![preview
                .view()
                .map_msg(move |msg| Msg::Preview(i.clone(), msg))]
        })]]
    }
}
