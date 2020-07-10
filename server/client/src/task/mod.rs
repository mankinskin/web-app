use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
};
use database::{
    Entry,
};
use rql::{
    *,
};
use crate::{
    root::{
        GMsg,
    },
    route::{self},
    preview::{self, Preview},
    entry::{
        self,
        TableItem,
    },
    config::{
        Component,
        View,
        Child,
    },
    editor::{
        Edit,
    },
};
use updatable::{
    Updatable,
};
use std::result::Result;
use async_trait::async_trait;

pub mod editor;
pub mod profile;
pub mod list;

#[derive(Clone)]
pub enum Msg {
    SetDescription(String),
    SetTitle(String),
    Entry(Box<entry::Msg<Task>>),
}
impl Component for Task {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::SetTitle(n) => {
                self.set_title(n);
            },
            Msg::SetDescription(d) => {
                self.set_description(d);
            },
            Msg::Entry(_) => {},
        }
    }
}
impl View for Task {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p![self.title()],
        ]
    }
}
#[async_trait(?Send)]
impl TableItem for Task {
    fn table_route() -> route::Route {
        route::Route::Home
    }
    fn entry_route(id: Id<Self>) -> route::Route {
        route::Route::Task(id)
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        api::get_tasks()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        api::get_task(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        api::delete_task(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String> {
        api::update_task(id, update)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        api::post_task(data)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}

impl Child<entry::Model<Self>> for Task {
    fn parent_msg(msg: Self::Msg) -> Option<entry::Msg<Self>> {
        match msg {
            Msg::Entry(msg) => Some(*msg),
            _ => None
        }
    }
}

impl Preview for Task {
    fn preview(&self) -> Node<Msg> {
        div![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
                    St::Cursor => "pointer",
            },
            simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Preview(Box::new(preview::Msg::Open))))),
            h3![
                style!{
                    St::Margin => "0",
                },
                self.title(),
            ],
            div![],
            p![
                style!{
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.subtasks().len(),

            p![
                style!{
                    St::Margin => "0",
                },
                "Assignees:"
            ],
            self.assignees().len(),
            button![
                simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
impl Edit for Task {
    fn edit(&self) -> Node<Msg> {
        form![
            label![
                "Title"
            ],
            input![
                attrs!{
                    At::Placeholder => "Title",
                    At::Value => self.title(),
                },
                input_ev(Ev::Input, Msg::SetTitle)
            ],
            label![
                "Description"
            ],
            textarea![
                attrs!{
                    At::Placeholder => "Description...",
                    At::Value => self.description(),
                },
                input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
