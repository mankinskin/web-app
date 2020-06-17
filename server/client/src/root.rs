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
    MutexGuard,
};

lazy_static! {
    static ref USER_SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
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
pub fn set_session(session: UserSession) {
    *USER_SESSION.lock().unwrap() = Some(session);
}
pub fn get_session() -> Option<UserSession> {
    USER_SESSION.lock().unwrap().clone()
}
pub fn end_session() {
    *USER_SESSION.lock().unwrap() = None;
}
#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    RouteChanged(Route),
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
        Msg::RouteChanged(route) => {
            model.page = page::Model::from(route);
            page::update(
                page::Msg::FetchData,
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
        },
    }
}
pub fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        GMsg::Root(msg) => {
            orders.send_msg(msg);
        },
        GMsg::SetSession(session) => {
            set_session(session.clone());
            storage::store_session(&session.clone());
        },
        GMsg::EndSession => {
            storage::delete_app_data();
            end_session()
        }
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
    Some(Msg::RouteChanged(Route::from(url.path)))
}
fn after_mount(url: Url, orders: &mut impl Orders<Msg, GMsg>) -> AfterMount<Model> {
    let route = Route::from(url.path);
    orders.send_msg(Msg::RouteChanged(route.clone()));
    if let Some(session) = storage::load_session() {
        orders.send_g_msg(GMsg::SetSession(session));
    }
    AfterMount::default()
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .routes(routes)
        .sink(sink)
        .after_mount(after_mount)
        .build_and_start();
}
