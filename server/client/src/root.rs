use seed::{
    *,
    prelude::*,
};
use crate::{
    *,
    config::{
        Component,
        Config,
        View,
    },
};
use api::{
    routes::{
        Route,
        Routable,
    },
};
use plans::{
    user::{
        UserSession,
    },
};

#[wasm_bindgen(start)]
pub fn render() {
    App::start("app",
               |url, orders| Config::<Model>::init(Route::from(url), orders),
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Clone, Default)]
pub struct Model {
    navbar: navbar::Model,
    page: page::Model,
}
impl Config<Model> for Route {
    fn into_model(self, orders: &mut impl Orders<Msg>) -> Model {
        orders.subscribe(Msg::UrlRequested)
              .subscribe(Msg::UrlChanged)
              .subscribe(|route| Msg::Page(page::Msg::GoTo(route)))
              .subscribe(|msg: Msg| msg);
        Model {
            navbar: Default::default(),
            page: Config::init(self, &mut orders.proxy(Msg::Page)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    UrlRequested(subs::UrlRequested),
    UrlChanged(subs::UrlChanged),
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetSession(UserSession),
    EndSession,
}
fn refresh_session() {
    if let None = api::auth::get_session() {
        if let Some(session) = api::auth::load_session() {
            api::auth::set_session(session);
        }
    }
}
pub fn go_to<R: Routable, Ms: 'static>(r: R, orders: &mut impl Orders<Ms>) {
    orders.notify(Msg::Page(page::Msg::GoTo(r.route())));
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        refresh_session();
        match msg {
            Msg::UrlChanged(subs::UrlChanged(url)) => {
                orders.send_msg(Msg::Page(page::Msg::GoTo(Route::from(url))));
            },
            Msg::UrlRequested(subs::UrlRequested(url, _request)) => {
                orders.send_msg(Msg::Page(page::Msg::GoTo(Route::from(url))));
            },
            Msg::SetSession(session) => {
                api::auth::set_session(session);
            },
            Msg::EndSession => {
                api::auth::end_session();
                self.page = page::Model::default();
            },
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
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.navbar.view().map_msg(Msg::NavBar),
            self.page.view().map_msg(Msg::Page),
        ]
    }
}
