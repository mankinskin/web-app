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
        View,
        Child,
    },
    root::{
        GMsg,
    },
    preview::{self, Preview},
    route::{
        self,
        Routable,
        Route,
    },
    editor::{
        Edit,
    },
};
use database::{
    Entry,
};
use std::result::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait TableItem: Clone + 'static + Child<Model<Self>> + Updatable {
    fn table_route() -> route::Route;
    fn entry_route(id: Id<Self>) -> route::Route;
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String>;
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String>;
    async fn get_all() -> Result<Vec<Entry<Self>>, String>;
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String>;
    async fn post(data: Self) -> Result<Id<Self>, String>;
}
#[derive(Clone)]
pub struct Model<T: TableItem> {
    pub entry: Entry<T>,
}
impl<T: TableItem + Component> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Model<T> {
        Model {
            entry,
        }
    }
}
impl<T: TableItem> Routable for Model<T>
    where Id<T>: Routable,
{
    fn route(&self) -> Route {
        self.entry.route()
    }
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
impl<T: TableItem + Component> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Refresh => {
                orders.perform_cmd(
                    T::get(self.entry.id).map(|res| Msg::Refreshed(res))
                );
            },
            Msg::Refreshed(res) => {
                match res {
                    Ok(r) =>
                        if let Some(entry) = r {
                            self.entry = entry;
                        },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Delete => {
                orders.perform_cmd(
                    T::delete(self.entry.id).map(|res| Msg::Deleted(res))
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
                    <T as TableItem>::update(self.entry.id, T::Update::from(self.entry.data.clone()))
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
                self.entry.data.update(msg.clone(), &mut orders.proxy(Msg::Data));
                T::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            Msg::Preview(_) => {
            },
        }
    }
}
impl<T: TableItem> Child<preview::Model<T>> for Model<T> {
    fn parent_msg(msg: Self::Msg) -> Option<preview::Msg<T>> {
        match msg {
            Msg::Preview(msg) => Some(*msg),
            _ => None
        }
    }
}
impl<T: TableItem + View> View for Model<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.entry.data.view().map_msg(Msg::Data)
    }
}
impl<T: TableItem + Edit> Edit for Model<T> {
    fn edit(&self) -> Node<Self::Msg> {
        self.entry.data.edit().map_msg(Msg::Data)
    }
}
impl<T: TableItem + Preview + Child<Model<T>>> Preview for Model<T> {
    fn preview(&self) -> Node<Self::Msg> {
        self.entry.data.preview().map_msg(Msg::Data)
    }
}
