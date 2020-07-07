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
};
use database::{
    Entry,
};
use std::result::Result;
use async_trait::async_trait;

//pub mod editor;
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
            a![
                attrs!{
                    At::Href => "";
                },
                self.name(),
                simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Preview(Box::new(preview::Msg::Open))))),
            ],
            p!["Preview"],
            button![
                simple_ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
            //button![
            //    simple_ev(Ev::Click, Msg::Task(task::Msg::Edit)),
            //    "Edit"
            //],
        ]
    }
}
