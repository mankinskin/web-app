use crate::{
    shared::{
        Route,
        subscriptions::Route as SubscriptionRoute,
    },
    page::{
        Page,
    },
};
use components::{
    Component,
    Init,
    Viewable,
    router::{
        self,
        Router,
        ToRoute,
    },
};
use seed::{
    prelude::*,
    *,
};
use enum_paths::{
    AsPath,
};
use app_model::auth::Route as AuthRoute;

#[derive(Debug)]
pub struct Navbar {
    router: Router<Route, Page>
}
impl std::ops::Deref for Navbar {
    type Target = Router<Route, Page>;
    fn deref(&self) -> &Self::Target {
        &self.router
    }
}
impl std::ops::DerefMut for Navbar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.router
    }
}
impl<T: ToRoute<Route>> Init<T> for Navbar {
    fn init(t: T, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            router: router::Router::init(t.to_route(), &mut orders.proxy(Msg::Router)),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Router(router::Msg<Route, Page>),
}
impl Component for Navbar {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Navbar Update");
        match msg {
            Msg::Router(msg) => self.router.update(msg, &mut orders.proxy(Msg::Router)),
        }
    }
}
impl Viewable for Navbar {
    fn view(&self) -> Node<Msg> {
        div![
            // TODO
            a!["Home", attrs! { At::Href => &format!("/{}", Route::Root.as_path()) }],
            a!["Subscriptions", attrs! { At::Href => &Route::Subscriptions(SubscriptionRoute::List).as_path() }],
            a!["Login", attrs! { At::Href => &Route::Auth(AuthRoute::Login).as_path() }],
            a!["Register", attrs! { At::Href => &Route::Auth(AuthRoute::Register).as_path() }],

            self.router.view().map_msg(Msg::Router)
        ]
    }
}
