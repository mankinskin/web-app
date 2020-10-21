use crate::{
    shared::Route,
    page::{
        self,
        *,
    },
};
use components::{
    Component,
    Init,
    Viewable,
};
use enum_paths::ParsePath;
use seed::{
    app::subs,
    prelude::*,
    *,
};
use tracing::debug;
use enum_paths::{
    AsPath,
};
use app_model::auth::Route as AuthRoute;

#[derive(Debug)]
pub struct Router {
    host: String,
    page: Page,
    url_changed_sub: SubHandle,
}
impl Router {
    pub fn go_to_url(&mut self, url: Url, orders: &mut impl Orders<Msg>) {
        //debug!("Go to Url");
        let route = if let Ok(route) = ParsePath::parse_path(&url.to_string()) {
            route
        } else {
            Route::Root
        };
        self.go_to_route(route, orders);
    }
    pub fn go_to_route(&mut self, route: Route, orders: &mut impl Orders<Msg>) {
        //debug!("Go to route");
        self.set_page(Page::init(route, &mut orders.proxy(Msg::Page)));
    }
    pub fn set_page(&mut self, page: Page) {
        debug!("Set page");
        self.page = page;
    }
}
impl Init<Url> for Router {
    fn init(url: Url, orders: &mut impl Orders<Msg>) -> Self {
        let route = if let Ok(route) = ParsePath::parse_path(&url.to_string()) {
            route
        } else {
            Route::Root
        };
        Self::init(route, orders)
    }
}
impl Init<Route> for Router {
    fn init(route: Route, orders: &mut impl Orders<Msg>) -> Self {
        let host = crate::get_host().unwrap();
        Self {
            host,
            page: Page::init(route, &mut orders.proxy(Msg::Page)),
            url_changed_sub: orders.subscribe_with_handle(Msg::UrlChanged),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Page(page::Msg),
    UrlChanged(subs::UrlChanged),
}
impl Component for Router {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Router Update");
        match msg {
            Msg::Page(msg) => self.page.update(msg, &mut orders.proxy(Msg::Page)),
            Msg::UrlChanged(subs::UrlChanged(url)) => {
                debug!("UrlChanged");
                self.go_to_url(url, orders);
            }
        }
    }
}
impl Viewable for Router {
    fn view(&self) -> Node<Msg> {
        div![
            // TODO
            a!["Home", attrs! { At::Href => &format!("/{}", Route::Root.as_path()) }],
            a!["Subscriptions", attrs! { At::Href => &Route::Subscriptions.as_path() }],
            a!["Login", attrs! { At::Href => &Route::Auth(AuthRoute::Login).as_path() }],
            a!["Register", attrs! { At::Href => &Route::Auth(AuthRoute::Register).as_path() }],
            self.page.view().map_msg(Msg::Page)
        ]
    }
}
