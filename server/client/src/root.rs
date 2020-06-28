use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
    route::*,
};
use plans::{
    user::*,
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
    SetPage(page::Config),
}
#[derive(Clone)]
pub enum GMsg {
    Root(Msg),
    SetSession(UserSession),
    EndSession,
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
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Page(msg) => {
            page::update(
                msg.clone(),
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
        },
        Msg::NavBar(msg) => {
            navbar::update(
                msg.clone(),
                &mut model.navbar,
                &mut orders.proxy(Msg::NavBar)
            );
        },
        Msg::SetPage(config) => {
            if let None = api::auth::get_session() {
                if let Some(session) = api::auth::load_session() {
                    api::auth::set_session(session);
                }
            }
            model.page = page::init(config, &mut orders.proxy(Msg::Page));
        },
    }
}
pub fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        GMsg::Root(msg) => {
            orders.send_msg(msg);
        },
        GMsg::SetSession(session) => {
            api::auth::set_session(session.clone());
        },
        GMsg::EndSession => {
            api::auth::end_session()
        },
    }
}
pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    div![
        navbar::view(&model.navbar)
            .map_msg(Msg::NavBar),
        page::view(&model.page)
            .map_msg(Msg::Page),
    ]
}
fn routes(url: Url) -> Option<Msg> {
    // needed to use Hrefs (because they only change the browser url)
    Some(Msg::SetPage(page::Config::from(Route::from(url.path()))))
}
#[derive(Clone)]
pub struct Config {
    navbar: navbar::Config,
    page: page::Config,
}
impl From<seed::Url> for Config {
    fn from(url: seed::Url) -> Self {
        Self {
            navbar: navbar::Config::default(),
            page: page::Config::from(url),
        }
    }
}
fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    Model {
        navbar: navbar::init(config.navbar, &mut orders.proxy(Msg::NavBar)),
        page: page::init(config.page, &mut orders.proxy(Msg::Page)),
    }
}

fn after_mount(url: seed::Url, orders: &mut impl Orders<Msg, GMsg>) -> AfterMount<Model> {
    AfterMount::new(init(Config::from(url), orders))
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .after_mount(after_mount)
        .routes(routes)
        .sink(sink)
        .build_and_start();
}
