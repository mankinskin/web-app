use seed::{
    prelude::*,
};
use rql::{
    Id,
};
use crate::{
    page,
    config::{
        Config,
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
pub struct Model<T: TableItem + Child<entry::Model<T>>>
{
    pub entry: entry::Model<T>,
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem + Child<entry::Model<T>>>
{
    Entry(entry::Msg<T>),
    Open,
}
impl<T: Component + TableItem + Child<entry::Model<T>>> Config<Model<T>> for Id<T>
{
    fn into_model(self, orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model {
            entry: Config::init(self, &mut orders.proxy(Msg::Entry)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>, GMsg>) {
    }
}
impl<T: Component + TableItem + Child<entry::Model<T>>> Config<Model<T>> for Entry<T>
{
    fn into_model(self, orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model {
            entry: Config::init(self, &mut orders.proxy(Msg::Entry)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>, GMsg>) {
    }
}
impl<T: Component + TableItem + Child<entry::Model<T>>> Component for Model<T>
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>, GMsg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::Entry)
                );
                entry::Model::<T>::parent_msg(msg).map(|msg| orders.send_msg(msg));
                //match msg {
                //    task::Msg::Edit => {
                //        page::go_to(route::Route::Task(model.task.task_id.clone()), orders);
                //    },
                //    _ => {}
                //}
            },
            Msg::Open => {
                page::go_to(self.entry.clone(), orders);
            },
        }
    }
}
impl<T: TableItem + Preview + Child<entry::Model<T>>> View for Model<T>
{
    fn view(&self) -> Node<Self::Msg> {
        self.entry.preview().map_msg(Msg::Entry)
    }
}
