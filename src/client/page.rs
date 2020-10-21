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
    SubscriptionList,
};

#[derive(Debug)]
pub enum Page {
    Auth(app_model::auth::Auth),
    SubscriptionList(SubscriptionList),
    Users,
    User,
    Root,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Auth(app_model::auth::Msg),
    SubscriptionList(subscriptions::Msg),
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
            Route::Subscriptions => Self::SubscriptionList(SubscriptionList::init((), &mut orders.proxy(Msg::SubscriptionList))),
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
            Self::SubscriptionList(list) => {
                if let Msg::SubscriptionList(msg) = msg {
                    list.update(msg, &mut orders.proxy(Msg::SubscriptionList));
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
            Self::SubscriptionList(list) => list.view().map_msg(Msg::SubscriptionList),
            _ => p!["Hello World!"],
        }
    }
}
