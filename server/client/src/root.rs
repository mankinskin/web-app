use crate::{
    navbar,
    page,
};
use app_model::{
    auth::{
        self,
        UserSession,
    },
    Route,
};
use components::{
    Component,
    Init,
    Viewable,
};
use database_table::Routable;
use enum_paths::ParsePath;
use seed::{
    prelude::*,
    *,
};

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start(
        "app",
        |url, orders| Model::init(Route::parse_path(&url.to_string()).unwrap(), orders),
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
}
#[derive(Clone, Default)]
pub struct Model {
    navbar: navbar::Model,
    page: page::Model,
}
impl Init<Route> for Model {
    fn init(route: Route, orders: &mut impl Orders<Msg>) -> Model {
        orders
            .subscribe(|msg: Msg| msg)
            .subscribe(Msg::UrlRequested)
            .subscribe(Msg::UrlChanged)
            .subscribe(|route| Msg::Page(page::Msg::GoTo(route)));
        Model {
            navbar: Default::default(),
            page: Init::init(route, &mut orders.proxy(Msg::Page)),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    UrlRequested(subs::UrlRequested),
    UrlChanged(subs::UrlChanged),
    NavBar(navbar::Msg),
    Page(page::Msg),
    SetSession(UserSession),
    EndSession,
}
fn refresh_session() {
    if let None = auth::session::get() {
        if let Some(session) = auth::session::load() {
            auth::session::set(session);
        }
    }
}
pub fn go_to<R: Routable, Ms: 'static>(_r: R, _orders: &mut impl Orders<Ms>) {
    //orders.notify(subs::UrlRequested::new(r.route().into()));
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        refresh_session();
        seed::log!(msg);
        match msg {
            Msg::UrlChanged(subs::UrlChanged(_url)) => {
                //orders.send_msg(Msg::Page(page::Msg::GoTo(Route::from(url))));
            }
            Msg::UrlRequested(subs::UrlRequested(_url, _request)) => {
                //orders.send_msg(Msg::Page(page::Msg::GoTo(Route::from(url))));
            }
            Msg::SetSession(session) => {
                auth::session::set(session);
            }
            Msg::EndSession => {
                auth::session::end();
                self.page = page::Model::default();
            }
            Msg::Page(msg) => {
                self.page.update(msg.clone(), &mut orders.proxy(Msg::Page));
            }
            Msg::NavBar(msg) => {
                self.navbar
                    .update(msg.clone(), &mut orders.proxy(Msg::NavBar));
            }
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.navbar.view().map_msg(Msg::NavBar),
            self.page.view().map_msg(Msg::Page),
        ]
    }
}
