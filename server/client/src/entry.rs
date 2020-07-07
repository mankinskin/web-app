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
        Child,
    },
    root::{
        GMsg,
    },
    preview::{self, Preview},
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
pub trait TableItem: Clone + 'static + Child<Model<Self>>
{
    fn table_route() -> route::Route;
    fn entry_route(id: Id<Self>) -> route::Route;
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String>;
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String>;
    async fn get_all() -> Result<Vec<Entry<Self>>, String>;
}

#[derive(Clone)]
pub struct Model<T: TableItem + Child<Model<T>>>
{
    pub id: Id<T>,
    pub data: Option<T>,
}
impl<T: TableItem + Component + Child<Model<T>>> Config<Model<T>> for Id<T>
{
    fn into_model(self, _orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model {
            id: self,
            data: None,
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg<T>, GMsg>) {
        orders.send_msg(Msg::Get);
    }
}
impl<T: TableItem + Component + Child<Model<T>>> Config<Model<T>> for Entry<T>
{
    fn into_model(self, _orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model {
            id: *self.id(),
            data: Some(self.data().clone()),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>, GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg<T: TableItem + Component>
{
    Get,
    Got(Result<Option<Entry<T>>, String>),

    Delete,
    Deleted(Result<Option<T>, String>),

    Data(<T as Component>::Msg),
    Preview(Box<preview::Msg<T>>),
}
impl<T: TableItem + Component> Component for Model<T>
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Get => {
                orders.perform_cmd(
                    T::get(self.id).map(|res| Msg::Got(res))
                );
            },
            Msg::Got(res) => {
                match res {
                    Ok(r) =>
                        if let Some(entry) = r {
                            self.id = entry.id().clone();
                            self.data = Some(entry.data().clone());
                        },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Delete => {
                orders.perform_cmd(
                    T::delete(self.id).map(|res| Msg::Deleted(res))
                );
            },
            Msg::Deleted(_res) => {
            },
            Msg::Data(msg) => {
                if let Some(data) = &mut self.data {
                    data.update(msg.clone(), &mut orders.proxy(Msg::Data));
                }
                T::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            Msg::Preview(_) => {
            },
        }
    }
}
impl<T: TableItem> Child<preview::Model<T>> for Model<T>
{
    fn parent_msg(msg: Self::Msg) -> Option<preview::Msg<T>> {
        match msg {
            Msg::Preview(msg) => Some(*msg),
            _ => None
        }
    }
}
impl<T: TableItem + View> View for Model<T>
{
    fn view(&self) -> Node<Self::Msg> {
        if let Some(data) = &self.data {
            data.view().map_msg(Msg::Data)
        } else {
            div![
                h1!["Entry"],
                p!["Loading..."],
            ]
        }
    }
}
impl<T: TableItem + Preview + Child<Model<T>>> Preview for Model<T>
{
    fn preview(&self) -> Node<Self::Msg> {
        match &self.data {
            Some(data) => {
                data.preview().map_msg(Msg::Data)
            },
            None => {
                div![
                    h1!["Preview"],
                    p!["Loading..."],
                ]
            },
        }
    }
}
