use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
        Config,
    },
    preview::{self, Preview},
    entry::{
        self,
    },
};
use database::{
    Entry,
};
use api::{
    TableItem,
};
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model<T: TableItem + Component> {
    previews: Vec<preview::Model<T>>,
}
impl<T: Component + TableItem + Default> Config<Model<T>> for Msg<T>
{
    fn into_model(self, _orders: &mut impl Orders<Msg<T>>) -> Model<T> {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg<T>>) {
        orders.send_msg(self);
    }
}
impl<T: Component + TableItem> From<Vec<Entry<T>>> for Model<T> {
    fn from(entries: Vec<Entry<T>>) -> Self {
        Self {
            previews: init_previews(entries),
        }
    }
}
fn init_previews<T: Component + TableItem>(entries: Vec<Entry<T>>) -> Vec<preview::Model<T>> {
    entries
        .iter()
        .cloned()
        .map(preview::Model::from)
        .collect()
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem> {
    GetAll,
    All(Result<Vec<Entry<T>>, String>),

    Preview(usize, preview::Msg<T>),
}
impl<T: Component + TableItem> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    T::get_all()
                        .map(|res| Msg::All(res))
                );
            },
            Msg::All(res) => {
                match res {
                    Ok(entries) => self.previews = init_previews(entries),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Preview(index, msg) => {
                self.previews[index].update(
                    msg.clone(),
                    &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
                );
                if let preview::Msg::Entry(entry::Msg::Deleted(_)) = msg {
                    self.previews.remove(index);
                }
            },
        }
    }
}
impl <T: Component + Preview + TableItem> View for Model<T> {
    fn view(&self) -> Node<Msg<T>> {
        div![
            ul![
                self.previews.iter().enumerate()
                    .map(|(i, preview)| li![
                         preview.view()
                            .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                    ])
            ]
        ]
    }
}
