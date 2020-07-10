use seed::{
    prelude::*,
};
use crate::{
    page,
    config::{
        Component,
        View,
        Child,
    },
    entry::{self, TableItem},
    root::{GMsg},
};
use database::{
    Entry,
};

pub trait Preview : View {
    fn preview(&self) -> Node<Self::Msg>;
}

#[derive(Clone)]
pub struct Model<T: TableItem + Child<entry::Model<T>>> {
    pub entry: entry::Model<T>,
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem + Child<entry::Model<T>>> {
    Entry(entry::Msg<T>),
    Open,
}
impl<T: Component + TableItem + Child<entry::Model<T>>> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Self {
        Model {
            entry: entry::Model::from(entry),
        }
    }
}
impl<T: Component + TableItem + Child<entry::Model<T>>> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>, GMsg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::Entry)
                );
                entry::Model::<T>::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            Msg::Open => {
                page::go_to(self.entry.clone(), orders);
            },
        }
    }
}
impl<T: TableItem + Preview + Child<entry::Model<T>>> View for Model<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.entry.preview().map_msg(Msg::Entry)
    }
}
