use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
    route::*,
    config::{
        Component,
        Config,
        View,
    },
};
use plans::{
    user::*,
};

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(move |msg, model, orders| model.update(msg, orders), Model::view)
        .after_mount(after_mount)
        .routes(routes)
        .sink(sink)
        .build_and_start();
}
fn after_mount(url: seed::Url, orders: &mut impl Orders<Msg, GMsg>) -> AfterMount<Model> {
    AfterMount::new(Config::init(Route::from(url), orders))
}
fn routes(url: Url) -> Option<Msg> {
    // needed to use Hrefs (because they only change the browser url)
    Some(Msg::SetPage(Route::from(url.path())))
}
#[derive(Clone, Default)]
pub struct Model {
    navbar: navbar::Model, // the navigation bar
    page: page::Model, // the current page
}
impl Config<Model> for Route {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            navbar: Default::default(),
            page: Config::init(self, &mut orders.proxy(Msg::Page)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetPage(Route),
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
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::Page(msg) => {
                self.page.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::Page)
                );
            },
            Msg::NavBar(msg) => {
                self.navbar.update(
                    msg.clone(),
                    &mut orders.proxy(Msg::NavBar)
                );
            },
            Msg::SetPage(route) => {
                if let None = api::auth::get_session() {
                    if let Some(session) = api::auth::load_session() {
                        api::auth::set_session(session);
                    }
                }
                self.page = Config::init(route, &mut orders.proxy(Msg::Page));
            },
        }
    }
}
#[derive(Clone)]
pub enum GMsg {
    Root(Msg),
    SetSession(UserSession),
    EndSession,
}
fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
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
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.navbar.view()
                .map_msg(Msg::NavBar),
            self.page.view()
                .map_msg(Msg::Page),
        ]
    }
}
