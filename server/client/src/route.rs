use seed::{
    self,
    prelude::*,
};
use crate::{
    root::{
        self,
        GMsg,
    },
    page,
};
#[derive(Clone, Debug)]
pub enum Route {
    Home,
    Login,
    Register,
    Users,
}
impl Route {
    pub fn path(&self) -> Vec<&str> {
        use Route::*;
        match self {
            Home => vec![],
            Login => vec!["login"],
            Register => vec!["register"],
            Users => vec!["users"],
        }
    }
}
impl From<Route> for seed::Url {
    fn from(route: Route) -> Self {
        route.path().into()
    }
}
impl From<Route> for page::Model {
    fn from(route: Route) -> Self {
        match route {
            Route::Home => Self::home(),
            Route::Login => Self::login(),
            Route::Register => Self::register(),
            Route::Users => Self::users(),
        }
    }
}
pub fn change_route<Ms: 'static>(route: Route, orders: &mut impl Orders<Ms, GMsg>) {
    seed::push_route(route.clone());
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(page::Model::from(route))));
}
