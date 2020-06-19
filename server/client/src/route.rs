use seed::{
    self,
    prelude::*,
};
use crate::{
    root::{
        self,
        GMsg,
    },
    fetched,
};
use rql::{
    *,
};
use plans::{
    user::*,
};
use std::str::{
    FromStr,
};
#[derive(Clone, Debug)]
pub enum Route {
    Home,
    Login,
    Register,
    Users,
    UserProfile(fetched::Query<User>),
    NotFound,
}
impl Into<Vec<String>> for Route {
    fn into(self) -> Vec<String> {
        match self {
            Route::Home => vec![],
            Route::Login => vec!["login".into()],
            Route::Register => vec!["register".into()],
            Route::Users => vec!["users".into()],
            Route::UserProfile(query) => vec!["users".into(), query.to_string()],
            Route::NotFound => vec![],
        }
    }
}
impl From<&[String]> for Route {
    fn from(path: &[String]) -> Self {
        if path.is_empty() {
            Route::Home
        } else {
            match &path[0][..] {
                "login" => Route::Login,
                "register" => Route::Register,
                "users" =>
                    if path.len() == 1 {
                        Route::Users
                    } else if path.len() == 2 {
                        match Id::from_str(&path[1]) {
                            Ok(id) => Route::UserProfile(fetched::Query::Id(id)),
                            Err(_e) => Route::NotFound,
                        }
                    } else {
                        Route::NotFound
                    },
                _ => Route::Home,
            }
        }
    }
}
impl From<Route> for seed::Url {
    fn from(route: Route) -> Self {
        Self::new().set_path(<Route as Into<Vec<String>>>::into(route))
    }
}
pub fn change_route<Ms: 'static>(route: Route, orders: &mut impl Orders<Ms, GMsg>) {
    seed::push_route(route.clone());
    orders.send_g_msg(GMsg::Root(root::Msg::RouteChanged(route)));
}
