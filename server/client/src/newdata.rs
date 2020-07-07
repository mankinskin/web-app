use seed::{
    prelude::*,
};
use rql::{
    Id,
};
use crate::{
    config::{
        Component,
        View,
    },
    route::{
        Routable,
    },
    root::{
        GMsg,
    },
    entry::{
        TableItem,
    },
};
use std::result::Result;

#[derive(Clone)]
pub struct Model<T> {
    pub data: T,
}
impl<T> From<T> for Model<T> {
    fn from(data: T) -> Self {
        Self {
            data,
        }
    }
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem>
    where Id<T>: Routable
{
    Post,
    Posted(Result<Id<T>, String>),
    Data(<T as Component>::Msg),
}
impl<T: Component + TableItem> Component for Model<T>
    where Id<T>: Routable
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Post => {
                //orders.perform_cmd(
                //    api::get_task(id)
                //        .map(|res| Msg::GotTask(res.map_err(|e| format!("{:?}", e))))
                //);
            },
            Msg::Posted(res) => {
                match res {
                    Ok(id) => { seed::log(id); },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Data(msg) => {
                self.data.update(msg, &mut orders.proxy(Msg::Data))
            },
        }
    }
}
impl<T: View + TableItem> View for Model<T>
    where Id<T>: Routable
{
    fn view(&self) -> Node<Self::Msg> {
        self.data.view().map_msg(Msg::Data)
    }
}
