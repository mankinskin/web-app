use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
};

#[derive(Clone)]
pub struct Model {
    navbar: navbar::Model, // the navigation bar
    page: page::Model, // the current page
}
impl Model {
    pub fn home_page() -> Self {
        Self {
            navbar: navbar::Model::default(),
            page: page::Model::home(),
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::home_page()
    }
}

#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetPage(page::Model),
}
impl From<page::Msg> for Msg {
    fn from(msg: page::Msg) -> Self {
        Self::Page(msg)
    }
}
impl From<navbar::Msg> for Msg {
    fn from(msg: navbar::Msg) -> Self {
        Self::NavBar(msg)
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetPage(page) => {
            model.page = page;
        },
        Msg::Page(msg) => {
            page::update(
                msg.clone(),
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
            match msg {
                _ => {},
            }
        },
        Msg::NavBar(msg) => {
            navbar::update(
                msg.clone(),
                &mut model.navbar,
                &mut orders.proxy(Msg::NavBar)
            );
            match msg {
                _ => {},
            }
        },
    }
}
pub fn view(model: &Model) -> impl View<Msg> {
    div![
        navbar::view(&model.navbar)
            .map_msg(Msg::NavBar),

        page::view(&model.page)
            .map_msg(Msg::Page),
    ]
}
