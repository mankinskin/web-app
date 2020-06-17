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

#[derive(Clone, Default)]
pub struct Model {
    session: Option<UserSession>, // session of login
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
impl Model {
    pub fn set_session(&mut self, session: UserSession) {
        self.session = Some(session);
    }
}
#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    RouteChanged(Route),
    SetSession(UserSession),
    EndSession,
}
#[derive(Clone)]
pub enum GMsg {
    Root(Msg),
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
        Msg::SetSession(session) => {
            model.set_session(session.clone());
            storage::store_session(&session.clone());
            navbar::update(
                navbar::Msg::SetSession(session.clone()),
                &mut model.navbar,
                &mut orders.proxy(Msg::NavBar)
            );
            page::update(
                page::Msg::SetSession(session),
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
        },
        Msg::EndSession => {
            storage::delete_app_data();
            navbar::update(
                navbar::Msg::EndSession,
                &mut model.navbar,
                &mut orders.proxy(Msg::NavBar)
            );
            page::update(
                page::Msg::EndSession,
                &mut model.page,
                &mut orders.proxy(Msg::Page)
            );
        }
    }
}
pub fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        GMsg::Root(msg) => {
            orders.send_msg(msg);
        }
    }
}
pub fn set_session<Ms: 'static>(session: UserSession, orders: &mut impl Orders<Ms, GMsg>) {
    orders.send_g_msg(GMsg::Root(root::Msg::SetSession(session)));
}
pub fn end_session<Ms: 'static>(orders: &mut impl Orders<Ms, GMsg>) {
    orders.send_g_msg(GMsg::Root(root::Msg::EndSession));
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
        orders.send_msg(Msg::SetSession(session));
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
