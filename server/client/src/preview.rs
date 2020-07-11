use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
        Child,
    },
    route::{
        Routable,
    },
    entry::{self, TableItem},
};
use database::{
    Entry,
};

pub trait Preview : Component {
    fn preview(&self) -> Node<Self::Msg>;
}
impl<T: Component + TableItem + Preview> Preview for Entry<T>
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
#[derive(Clone)]
pub struct Model<T: TableItem + Child<Entry<T>>> {
    pub entry: Entry<T>,
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem + Child<Entry<T>>> {
    Entry(entry::Msg<T>),
    //Open,
}
impl<T: Component + TableItem + Child<Entry<T>>> From<Entry<T>> for Model<T> {
    fn from(entry: Entry<T>) -> Self {
        Model {
            entry,
        }
    }
}
impl<T: Component + TableItem + Child<Entry<T>>> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::Entry)
                );
                Entry::<T>::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            //Msg::Open => {
            //    page::go_to(self.entry.clone(), orders);
            //},
        }
    }
}
impl<T: TableItem + Preview + Child<Entry<T>>> View for Model<T> {
    fn view(&self) -> Node<Self::Msg> {
        self.entry.preview().map_msg(Msg::Entry)
    }
}
