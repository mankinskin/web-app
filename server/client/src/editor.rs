use seed::{
    *,
    prelude::*,
};
use rql::{
    Id,
};
use database::{
    Entry,
};
use crate::{
    config::{
        Config,
        Component,
    },
    entry::{
        self,
        *,
    },
    newdata,
    remote,
};
pub trait Edit : Component {
    fn edit(&self) -> Node<Self::Msg>;
}
impl<T: TableItem + Edit> Edit for Entry<T> {
    fn edit(&self) -> Node<Self::Msg> {
        self.data.edit().map_msg(entry::Msg::Data)
    }
}
#[derive(Clone)]
pub enum Model<T: TableItem> {
    Remote(remote::Model<T>),
    New(newdata::Model<T>),
}
impl<T: TableItem + Default> Default for Model<T> {
    fn default() -> Self {
        Self::New(Default::default())
    }
}
impl<T: TableItem> From<Entry<T>> for Model<T> {
    fn from(model: Entry<T>) -> Self {
        Self::from(remote::Model::from(model))
    }
}
impl<T: TableItem> From<remote::Model<T>> for Model<T> {
    fn from(model: remote::Model<T>) -> Self {
        Self::Remote(model)
    }
}
impl<T: TableItem> Config<Model<T>> for Id<T> {
    fn into_model(self, orders: &mut impl Orders<Msg<T>>) -> Model<T> {
        Model::Remote(Config::init(self, &mut orders.proxy(Msg::Remote)))
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>>) {
    }
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem> {
    Cancel,
    Submit,
    New(newdata::Msg<T>),
    Remote(remote::Msg<T>),
}
impl<T: Component + TableItem> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::Cancel => {},
            Msg::Submit => {
                match self {
                    Model::New(new) =>
                        new.update(
                            newdata::Msg::Post,
                            &mut orders.proxy(Msg::New)
                            ),
                    Model::Remote(remote) =>
                        remote.update(
                            remote::Msg::Entry(entry::Msg::Update),
                            &mut orders.proxy(Msg::Remote)
                            ),
                }
            },
            Msg::New(msg) => {
                match self {
                    Model::New(new) => new.update(msg, &mut orders.proxy(Msg::New)),
                    _ => {},
                }
            },
            Msg::Remote(msg) => {
                match self {
                    Model::Remote(remote) => remote.update(msg, &mut orders.proxy(Msg::Remote)),
                    _ => {},
                }
            },
        }
    }
}
impl<T: Component + TableItem + Edit> Edit for Model<T> {
    fn edit(&self) -> Node<Msg<T>> {
        form![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
            },
            // Cancel Button
            button![ev(Ev::Click, |_| Msg::<T>::Cancel), "Cancel"],
            match self {
                Model::New(new) =>
                    div![
                        h1!["New"],
                        new.edit().map_msg(Msg::New),
                        // Submit Button
                        button![
                            attrs!{
                                At::Type => "submit",
                            },
                            "Create"
                        ],
                    ],
                Model::Remote(entry) =>
                    div![
                        h1!["Edit"],
                        entry.edit().map_msg(Msg::Remote),
                        button![
                            attrs!{
                                At::Type => "submit",
                            },
                            "Update"
                        ],
                    ],
            },
            ev(Ev::Submit, |ev| {
                ev.prevent_default();
                Msg::<T>::Submit
            }),
        ]
    }
}
