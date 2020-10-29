use crate::{
    entry,
    newdata,
    remote,
    Component,
    Init,
    Viewable,
};
use database_table::{
    Entry,
    RemoteTable,
};
use rql::Id;
use seed::{
    prelude::*,
    *,
};
use std::fmt::Debug;

pub trait Edit: Component {
    fn edit(&self) -> Node<Self::Msg>;
}
impl<T: RemoteTable + Edit + Debug> Edit for Entry<T> {
    fn edit(&self) -> Node<Self::Msg> {
        self.data.edit().map_msg(entry::Msg::Data)
    }
}
#[derive(Debug, Clone)]
pub enum Editor<T: RemoteTable> {
    Remote(remote::Remote<T>),
    New(newdata::NewData<T>),
}
impl<T: RemoteTable + Default> Default for Editor<T> {
    fn default() -> Self {
        Self::New(Default::default())
    }
}
impl<T: RemoteTable + Component> From<Entry<T>> for Editor<T> {
    fn from(model: Entry<T>) -> Self {
        Self::from(remote::Remote::from(model))
    }
}
impl<T: RemoteTable> From<remote::Remote<T>> for Editor<T> {
    fn from(model: remote::Remote<T>) -> Self {
        Self::Remote(model)
    }
}
impl<T: RemoteTable + Component + Debug + Clone> Init<Id<T>> for Editor<T> {
    fn init(id: Id<T>, orders: &mut impl Orders<Msg<T>>) -> Self {
        Self::Remote(Init::init(id, &mut orders.proxy(Msg::Remote)))
    }
}
#[derive(Debug)]
pub enum Msg<T: Component + RemoteTable + std::fmt::Debug> {
    Cancel,
    Submit,
    New(newdata::Msg<T>),
    Remote(remote::Msg<T>),
}
impl<T: Component + RemoteTable + Debug + Clone> Component for Editor<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::Cancel => {}
            Msg::Submit => {
                match self {
                    Self::New(new) => new.update(newdata::Msg::Post, &mut orders.proxy(Msg::New)),
                    Self::Remote(remote) => {
                        remote.update(
                            remote::Msg::Entry(entry::Msg::Update),
                            &mut orders.proxy(Msg::Remote),
                        )
                    }
                }
            }
            Msg::New(msg) => {
                match self {
                    Self::New(new) => new.update(msg, &mut orders.proxy(Msg::New)),
                    _ => {}
                }
            }
            Msg::Remote(msg) => {
                match self {
                    Self::Remote(remote) => remote.update(msg, &mut orders.proxy(Msg::Remote)),
                    _ => {}
                }
            }
        }
    }
}
impl<T: Component + RemoteTable + Edit + Debug + Clone> Viewable for Editor<T> {
    fn view(&self) -> Node<Msg<T>> {
        form![
            style! {
                St::Display => "grid",
                St::GridTemplateColumns => "1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
            },
            // Cancel Button
            button![ev(Ev::Click, |_| Msg::<T>::Cancel), "Cancel"],
            match self {
                Self::New(new) =>
                    div![
                        h1!["New"],
                        new.edit().map_msg(Msg::New),
                        // Submit Button
                        button![
                            attrs! {
                                At::Type => "submit",
                            },
                            "Create"
                        ],
                    ],
                Self::Remote(entry) =>
                    div![
                        h1!["Edit"],
                        entry.edit().map_msg(Msg::Remote),
                        button![
                            attrs! {
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
