use crate::{
    Component,
    Init,
    Viewable,
};
use enum_paths::{
    ParsePath,
    AsPath,
};
use seed::{
    app::subs,
    prelude::*,
    *,
};
use tracing::debug;
use std::fmt::Debug;

pub trait Route : AsPath + ParsePath + Default + Debug + Clone + 'static {
    fn parse_or_default<T: AsPath>(t: T) -> Self {
        if let Ok(r) = ParsePath::parse_path(&t.as_path()) {
            r
        } else {
            Self::default()
        }
    }
}

pub trait ToRoute<R: Route> : AsPath + Sized {
    fn to_route(self) -> R {
        R::parse_or_default(self.as_path())
    }
}
impl<T: AsPath, R: Route> ToRoute<R> for T {
    fn to_route(self) -> R {
        R::parse_or_default(self)
    }
}

pub trait Page<R: Route> : Init<R> + Component + Viewable + Debug + 'static {}
impl<R: Route, P: Init<R> + Component + Viewable + Debug + 'static> Page<R> for P {}

pub fn get_host() -> Result<String, JsValue> {
    web_sys::window().unwrap().location().host()
}

#[derive(Debug)]
pub struct Router<R: Route, P: Page<R>> {
    host: String,
    page: P,
    url_changed_sub: SubHandle,
    _ty: std::marker::PhantomData<R>,
}
impl<R: Route, P: Page<R>> Router<R, P> {
    pub fn go_to_url(&mut self, url: Url, orders: &mut impl Orders<Msg<R, P>>) {
        //debug!("Go to Url");
        self.go_to_route(R::parse_or_default(url), orders);
    }
    pub fn go_to_route(&mut self, route: R, orders: &mut impl Orders<Msg<R, P>>) {
        //debug!("Go to route");
        self.set_page(P::init(route, &mut orders.proxy(Msg::Page)));
    }
    pub fn set_page(&mut self, page: P) {
        debug!("Set page");
        self.page = page;
    }
}
impl<R: Route, P: Page<R>> Init<Url> for Router<R, P> {
    fn init(url: Url, orders: &mut impl Orders<Self::Msg>) -> Self {
        Self::init(R::parse_or_default(url), orders)
    }
}
impl<R: Route, P: Page<R>> Init<R> for Router<R, P> {
    fn init(route: R, orders: &mut impl Orders<Self::Msg>) -> Self {
        Self {
            host: get_host().unwrap(),
            page: P::init(route, &mut orders.proxy(Msg::Page)),
            url_changed_sub: orders.subscribe_with_handle(Self::Msg::UrlChanged),
            _ty: Default::default(),
        }
    }
}
#[derive(Debug)]
pub enum Msg<R: Route, P: Page<R>> {
    Page(<P as Component>::Msg),
    UrlChanged(subs::UrlChanged),
    GoTo(R),
}
impl<R: Route, P: Page<R>> Clone for Msg<R, P> {
    fn clone(&self) -> Self {
        match self {
            Self::Page(msg) => Self::Page(msg.clone()),
            Self::UrlChanged(msg) => Self::UrlChanged(msg.clone()),
            Self::GoTo(route) => Self::GoTo(route.clone()),
        }
    }
}
impl<R: Route, P: Page<R>> Component for Router<R, P> {
    type Msg = Msg<R, P>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        debug!("Router Update");
        match msg {
            Msg::Page(msg) => self.page.update(msg, &mut orders.proxy(Msg::Page)),
            Msg::UrlChanged(subs::UrlChanged(url)) => {
                debug!("UrlChanged");
                self.go_to_url(url, orders);
            },
            Msg::GoTo(route) => self.go_to_route(route, orders),
        }
    }
}
impl<R: Route, P: Page<R>> Viewable for Router<R, P> {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.page.view().map_msg(Msg::Page)
        ]
    }
}
