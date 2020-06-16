use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
};

#[derive(Clone, Default)]
pub struct Model {
    navbar: navbar::Model, // the navigation bar
    page: page::Model, // the current page
}

#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetPage(page::Model),
    FetchData,
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
            orders.send_msg(Msg::FetchData);
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
        Msg::FetchData => {
            page::update(
                page::Msg::FetchData,
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
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

fn routes(url: Url) -> Option<Msg> {
    if url.path.is_empty() {
        Some(Msg::SetPage(page::Model::home()))
    } else {
        match &url.path[0][..] {
            "login" => Some(Msg::SetPage(page::Model::login())),
            "register" => Some(Msg::SetPage(page::Model::register())),
            "users" => Some(Msg::SetPage(page::Model::users())),
            _ => Some(Msg::SetPage(page::Model::home())),
        }
    }
}
fn after_mount(_url: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders.send_msg(Msg::FetchData);
    AfterMount::default()
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .routes(routes)
        .after_mount(after_mount)
        .build_and_start();
}
