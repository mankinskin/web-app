use super::*;
use components::{
    Init,
    Component,
    auth::Auth,
};
use seed::{
    *,
    prelude::*,
};
use tracing::{
    debug,
};

#[derive(Clone, Debug)]
pub enum Msg {
    Auth(components::auth::Msg),
    Chart(crate::chart::Msg),
}
#[derive(Debug)]
pub enum Page {
    Auth(components::auth::Auth),
    Chart(crate::chart::Chart),
    Root,
}
impl Page {
    pub fn go_to(&mut self, route: BaseRoute, orders: &mut impl Orders<Msg>) {
        debug!("Goto Page");
        *self = Init::init(route, orders);
    }
}
impl Init<BaseRoute> for Page {
    fn init(route: BaseRoute, orders: &mut impl Orders<Msg>) -> Self {
        debug!("Init Page");
        match route {
            BaseRoute::Auth(auth_route) =>
                Self::Auth(Auth::init(auth_route, &mut orders.proxy(Msg::Auth))),
            BaseRoute::Chart =>
                Self::Chart(crate::chart::Chart::init((), &mut orders.proxy(Msg::Chart))),
            BaseRoute::Root => Self::Root,
        }
    }
}
impl Component for Page {
    type Msg = Msg; 
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        debug!("Page update");
        match self {
            Self::Auth(auth) =>
                if let Msg::Auth(msg) = msg {
                    auth.update(msg, &mut orders.proxy(Msg::Auth));
                },
            Self::Chart(chart) =>
                if let Msg::Chart(msg) = msg {
                    chart.update(msg, &mut orders.proxy(Msg::Chart));
                },
            Self::Root => {}
        }
    }
}
impl Viewable for Page {
    fn view(&self) -> Node<Msg> {
        match self {
            Self::Auth(auth) => auth.view().map_msg(Msg::Auth),
            Self::Chart(chart) => chart.view().map_msg(Msg::Chart),
            Self::Root => p!["Hello World!"],
        }
    }
}
