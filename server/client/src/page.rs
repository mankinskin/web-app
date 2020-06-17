use crate::*;
use seed::{
    *,
    prelude::*,
};
use rql::{
    *,
};
use plans::{
    user::*,
};
use crate::{
    route::{
        Route,
    },
    root::{
        GMsg,
    },
};
#[derive(Clone)]
pub enum Model {
    Login(login::Model),
    Register(register::Model),
    Home(home::Model),
    UserProfile(users::profile::Model),
    Users(users::Model),
    NotFound,
}
impl Default for Model {
    fn default() -> Self {
        Self::home()
    }
}
impl From<Route> for Model {
    fn from(route: Route) -> Self {
        match route {
            Route::Home => Self::home(),
            Route::Login => Self::login(),
            Route::Register => Self::register(),
            Route::Users => Self::users(),
            Route::UserProfile(id) => Self::profile(id),
            Route::NotFound => Self::not_found(),
        }
    }
}
impl From<home::Model> for Model {
    fn from(model: home::Model) -> Self {
        Self::Home(model)
    }
}
impl From<users::profile::Model> for Model {
    fn from(model: users::profile::Model) -> Self {
        Self::UserProfile(model)
    }
}
impl From<login::Model> for Model {
    fn from(model: login::Model) -> Self {
        Self::Login(model)
    }
}
impl From<register::Model> for Model {
    fn from(model: register::Model) -> Self {
        Self::Register(model)
    }
}
impl From<users::Model> for Model {
    fn from(model: users::Model) -> Self {
        Self::Users(model)
    }
}
impl Model {
    pub fn home() -> Self {
        Self::Home(home::Model::default())
    }
    pub fn users() -> Self {
        Self::Users(users::Model::fetch_all())
    }
    pub fn profile(id: Id<User>) -> Self {
        Self::UserProfile(users::user::Model::from(id).profile())
    }
    pub fn login() -> Self {
        Self::Login(login::Model::default())
    }
    pub fn register() -> Self {
        Self::Register(register::Model::default())
    }
    pub fn not_found() -> Self {
        Self::NotFound
    }
    pub fn route(&self) -> Route {
        match self {
            Self::UserProfile(profile) => Route::UserProfile(profile.user.user_id),
            Self::Users(_) => Route::Users,
            Self::Login(_) => Route::Login,
            Self::Register(_) => Route::Register,
            Self::Home(_) | Self::NotFound => Route::Home,
        }
    }
}
pub fn go_to<M: Into<page::Model> + Clone, Ms: 'static>(model: M, orders: &mut impl Orders<Ms, GMsg>) {
    let page: page::Model = model.into();
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(page)));
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    UserProfile(users::profile::Msg),
    Login(login::Msg),
    Register(register::Msg),
    Users(users::Msg),
    FetchData,
}
impl From<home::Msg> for Msg {
    fn from(msg: home::Msg) -> Self {
        Self::Home(msg)
    }
}
impl From<users::profile::Msg> for Msg {
    fn from(msg: users::profile::Msg) -> Self {
        Self::UserProfile(msg)
    }
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
impl From<register::Msg> for Msg {
    fn from(msg: register::Msg) -> Self {
        Self::Register(msg)
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match model {
        Model::Home(home) => {
            match msg {
                Msg::Home(msg) => {
                    home::update(
                        msg,
                        home,
                        &mut orders.proxy(Msg::Home)
                    );
                },
                _ => {}
            }
        },
        Model::Login(login) => {
            match msg {
                Msg::Login(msg) => {
                    login::update(
                        msg,
                        login,
                        &mut orders.proxy(Msg::Login)
                    );
                },
                _ => {}
            }
        },
        Model::Register(register) => {
            match msg {
                Msg::Register(msg) => {
                    register::update(
                        msg,
                        register,
                        &mut orders.proxy(Msg::Register)
                    );
                },
                _ => {}
            }
        },
        Model::Users(users) => {
            match msg {
                Msg::Users(msg) => {
                    users::update(
                        msg,
                        users,
                        &mut orders.proxy(Msg::Users)
                    );
                },
                Msg::FetchData => {
                    users::update(
                        users::Msg::FetchUsers,
                        users,
                        &mut orders.proxy(Msg::Users)
                    );
                }
                _ => {}
            }
        },
        Model::UserProfile(profile) => {
            match msg {
                Msg::UserProfile(msg) => {
                    users::profile::update(
                        msg,
                        profile,
                        &mut orders.proxy(Msg::UserProfile)
                    );
                },
                Msg::FetchData => {
                    users::profile::update(
                        users::profile::Msg::User(users::user::Msg::FetchUser),
                        profile,
                        &mut orders.proxy(Msg::UserProfile)
                    );
                }
                _ => {}
            }
        },
        _ => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::Home(home) =>
            home::view(&home)
                .map_msg(Msg::Home),
        Model::UserProfile(profile) =>
            users::profile::view(&profile)
                .map_msg(Msg::UserProfile),
        Model::Login(login) =>
            login::view(&login)
                .map_msg(Msg::Login),
        Model::Register(register) =>
            register::view(&register)
                .map_msg(Msg::Register),
        Model::Users(users) =>
            users::view(&users)
                .map_msg(Msg::Users),
        Model::NotFound =>
            div!["Not Found"]
    }
}
