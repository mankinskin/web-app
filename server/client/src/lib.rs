extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate http;
extern crate anyhow;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate rql;
extern crate plans;
extern crate budget;
extern crate updatable;
extern crate database;

use seed::{
    *,
    prelude::*,
};
pub mod login;
pub mod register;
pub mod home;

#[derive(Default)]
struct Model {
    login: login::Model,
    register: register::Model,
    home: home::Model,
    page: Page,
}

#[derive(Clone)]
enum Page {
    Login,
    Register,
    Home,
}
impl Default for Page {
    fn default() -> Self {
        Page::Home
    }
}
#[derive(Clone)]
enum Msg {
    Login(login::Msg),
    Register(register::Msg),
    Home(home::Msg),
    SetPage(Page),
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
impl From<register::Msg> for Msg {
    fn from(msg: register::Msg) -> Self {
        Self::Register(msg)
    }
}
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(msg) => {
            login::update(msg.clone(), &mut model.login, &mut orders.proxy(Msg::Login));
            match msg {
                login::Msg::Register => {
                    orders.send_msg(Msg::SetPage(Page::Register));
                },
                _ => {},
            }
        },
        Msg::Register(msg) => {
            register::update(msg.clone(), &mut model.register, &mut orders.proxy(Msg::Register));
            match msg {
                register::Msg::Login => {
                    orders.send_msg(Msg::SetPage(Page::Login));
                },
                _ => {},
            }
        },
        Msg::Home(msg) => {
            home::update(msg.clone(), &mut model.home, &mut orders.proxy(Msg::Home));
            match msg {
                _ => {},
            }
        },
        Msg::SetPage(page) => {
            model.page = page;
        },
    }
}
fn view(model: &Model) -> impl View<Msg> {
    div![
        match model.page {
            Page::Home =>
                home::view(&model.home)
                    .map_msg(Msg::Home),
            Page::Login =>
                login::view(&model.login)
                    .map_msg(Msg::Login),
            Page::Register =>
                register::view(&model.register)
                    .map_msg(Msg::Register),
        }
    ]
}
fn routes(url: Url) -> Option<Msg> {
    let path = url.path.join("/");
    match &path[..] {
        "" => Some(Msg::SetPage(Page::Home)),
        "login" => Some(Msg::SetPage(Page::Login)),
        "register" => Some(Msg::SetPage(Page::Register)),
        _ => Some(Msg::SetPage(Page::Register)),
    }
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .routes(routes)
        .build_and_start();
}
