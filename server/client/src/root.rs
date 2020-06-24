use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
    route::*,
    storage,
};
use plans::{
    user::*,
};
use std::sync::{
    Mutex,
};

lazy_static! {
    static ref USER_SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
}
pub fn set_session(session: UserSession) {
    *USER_SESSION.lock().unwrap() = Some(session);
}
pub fn get_session() -> Option<UserSession> {
    USER_SESSION.lock().unwrap().clone()
}
pub fn end_session() {
    *USER_SESSION.lock().unwrap() = None;
}
#[derive(Clone, Default)]
pub struct Model {
    navbar: navbar::Model, // the navigation bar
    page: page::Model, // the current page
}
impl From<Route> for Model {
    fn from(route: Route) -> Self {
        Self {
            page: page::Model::from(route),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetPage(page::Model),
    Fetch,
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
        Msg::Fetch => {
            page::update(
                page::Msg::Fetch,
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
        },
        Msg::SetPage(page) => {
            model.page = page;
            orders.send_msg(Msg::Fetch);
        },
    }
}
pub fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        GMsg::Root(msg) => {
            orders.send_msg(msg);
        },
        GMsg::SetSession(session) => {
            seed::log!("Setting session...");
            set_session(session.clone());
            storage::store_session(&session.clone());
            orders.send_msg(Msg::Fetch);
        },
        GMsg::EndSession => {
            seed::log!("ending session");
            storage::clean_storage();
            end_session()
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
    Some(Msg::SetPage(page::Model::from(Route::from(url.path()))))
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .routes(routes)
        .sink(sink)
        .build_and_start();
}
