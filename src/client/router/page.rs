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
use crate::chart::{
    self,
    Chart,
};
use app_model::route::{
    Route,
};

#[derive(Clone, Debug)]
pub enum Msg {
    Auth(components::auth::Msg),
    Chart(chart::Msg),
}
#[derive(Debug)]
pub enum Page {
    Auth(components::auth::Auth),
    Chart(Chart),
    Users,
    User,
    Projects,
    Project,
    Task,
    Root,
}
impl Init<Route> for Page {
    fn init(route: Route, orders: &mut impl Orders<Msg>) -> Self {
        debug!("Init Page");
        match route {
            Route::Auth(auth_route) =>
                Self::Auth(Auth::init(auth_route, &mut orders.proxy(Msg::Auth))),
            Route::Chart =>
                Self::Chart(Chart::init((), &mut orders.proxy(Msg::Chart))),
            Route::Project(_id) => Self::Project,
            Route::Projects => Self::Projects,
            Route::User(_id) => Self::User,
            Route::Users => Self::Users,
            Route::Task(_id) => Self::Task,
            Route::Root => Self::Root,
        }
    }
}
impl Component for Page {
    type Msg = Msg; 
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Page update");
        match self {
            Self::Auth(auth) =>
                if let Msg::Auth(msg) = msg {
                    auth.update(msg, &mut orders.proxy(Msg::Auth));
                },
            Self::Chart(chart) =>
                if let Msg::Chart(msg) = msg {
                    chart.update(msg, &mut orders.proxy(Msg::Chart));
                },
            Self::Users => {}
            Self::User => {}
            Self::Projects => {}
            Self::Project => {}
            Self::Task => {}
            Self::Root => {}
        }
    }
}
impl Viewable for Page {
    fn view(&self) -> Node<Msg> {
        match self {
            Self::Auth(auth) => auth.view().map_msg(Msg::Auth),
            Self::Chart(chart) => chart.view().map_msg(Msg::Chart),
            _ => p!["Hello World!"],
        }
    }
}
