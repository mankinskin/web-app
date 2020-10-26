use super::*;
use app_model::{
    auth::{
        Auth,
    },
    user::Route as UserRoute,
};
use crate::shared::{
    Route,
};
use components::{
    Component,
    Init,
};
use seed::{
    prelude::*,
    *,
};
use tracing::debug;
use crate::subscriptions::{
    self,
    Subscriptions,
};

#[derive(Debug)]
pub enum Page {
    Auth(app_model::auth::Auth),
    Subscriptions(Subscriptions),
    Users,
    User,
    Root,
}
#[derive(Debug)]
pub enum Msg {
    Auth(app_model::auth::Msg),
    Subscriptions(subscriptions::Msg),
}
impl Init<UserRoute> for Page {
    fn init(route: UserRoute, _orders: &mut impl Orders<Msg>) -> Self {
        match route {
            UserRoute::User(_id) => Self::User,
            UserRoute::Users => Self::Users,
        }
    }
}
impl Init<Route> for Page {
    fn init(route: Route, orders: &mut impl Orders<Msg>) -> Self {
        debug!("Init Page");
        match route {
            Route::Auth(auth_route) => {
                Self::Auth(Auth::init(auth_route, &mut orders.proxy(Msg::Auth)))
            }
            Route::Subscriptions(route) => Self::Subscriptions(Subscriptions::init(route, &mut orders.proxy(Msg::Subscriptions))),
            Route::User(route) => Self::init(route, orders),
            Route::Root => Self::Root,
        }
    }
}
impl Component for Page {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Page update");
        match self {
            Self::Auth(auth) => {
                if let Msg::Auth(msg) = msg {
                    auth.update(msg, &mut orders.proxy(Msg::Auth));
                }
            }
            Self::Subscriptions(list) => {
                if let Msg::Subscriptions(msg) = msg {
                    list.update(msg, &mut orders.proxy(Msg::Subscriptions));
                }
            }
            Self::Users => {}
            Self::User => {}
            Self::Root => {}
        }
    }
}
impl Viewable for Page {
    fn view(&self) -> Node<Msg> {
        match self {
            Self::Auth(auth) => auth.view().map_msg(Msg::Auth),
            Self::Subscriptions(list) => list.view().map_msg(Msg::Subscriptions),
            _ => p!["Hello World!"],
        }
    }
}
