use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
    },
    entry::{
        self,
    },
};
use api::{
    routes::{
        Routable,
    },
    TableItem,
};
use database::{
    Entry,
};
use std::fmt::Debug;

pub trait Preview : Component {
    fn preview(&self) -> Node<Self::Msg>;
}
impl<T: Component + TableItem + Preview + Debug> Preview for Entry<T>
{
    fn preview(&self) -> Node<Self::Msg> {
        div![
            attrs!{
                At::Href => &self.id.route().to_string();
            },
            self.data.preview().map_msg(entry::Msg::Data)
        ]
    }
}
#[derive(Clone, Debug)]
pub struct Model<T: TableItem + Debug> {
    pub entry: Entry<T>,
}
#[derive(Clone, Debug)]
pub enum Msg<T: Component + TableItem + Debug> {
    Entry(entry::Msg<T>),
    //Open,
}
impl<T: Component + TableItem + Debug> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Self {
        Model {
            entry,
        }
    }
}
impl<T: Component + TableItem + Debug> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::Entry)
                );
                //Entry::<T>::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            //Msg::Open => {
            //    page::go_to(self.entry.clone(), orders);
            //},
        }
    }
}
impl<T: TableItem + Preview + Debug> View for Model<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.entry.preview().map_msg(Msg::Entry)
    }
}
