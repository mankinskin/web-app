use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
};
use rql::{
    *,
};
use crate::{
    root::{
        GMsg,
    },
    route::{
        self,
    },
    preview::{
        self,
        Preview,
    },
    editor::{
        Edit,
    },
    entry::{
        self,
        TableItem,
    },
    config::{
        Component,
        View,
        Child,
    },
};
use updatable::{
    Updatable,
};
use database::{
    Entry,
};
use std::result::Result;
use async_trait::async_trait;

pub mod editor;
pub mod list;
pub mod profile;

#[async_trait(?Send)]
impl TableItem for Project {
    fn table_route() -> route::Route {
        route::Route::Projects
    }
    fn entry_route(id: Id<Self>) -> route::Route {
        route::Route::Project(id)
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, String> {
        api::get_projects()
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, String> {
        api::get_project(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, String> {
        api::delete_project(id)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Result<Option<Self>, String> {
        api::update_project(id, update)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
    async fn post(data: Self) -> Result<Id<Self>, String> {
        api::post_project(data)
            .map(|res| res.map_err(|e| format!("{:?}", e)))
            .await
    }
}

#[derive(Clone)]
pub enum Msg {
    SetName(String),
    SetDescription(String),
    Entry(Box<entry::Msg<Project>>),
}
impl Component for Project {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::SetName(n) => {
                self.set_name(n);
            },
            Msg::SetDescription(d) => {
                self.set_description(d);
            },
            Msg::Entry(_) => {}
        }
    }
}
impl View for Project {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p![self.name()],
        ]
    }
}
impl Child<entry::Model<Self>> for Project {
    fn parent_msg(msg: Self::Msg) -> Option<entry::Msg<Self>> {
        match msg {
            Msg::Entry(msg) => Some(*msg),
            _ => None
        }
    }
}
impl Preview for Project {
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
                self.name(),
            ],
            div![],

            p![
                style!{
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.tasks().len(),

            p![
                style!{
                    St::Margin => "0",
                },
                "Members:"
            ],
            self.members().len(),

            button![
                simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
impl Edit for Project {
    fn edit(&self) -> Node<Msg> {
        form![
            label![
                "Name"
            ],
            input![
                attrs!{
                    At::Placeholder => "Name",
                    At::Value => self.name(),
                },
                input_ev(Ev::Input, Msg::SetName)
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
