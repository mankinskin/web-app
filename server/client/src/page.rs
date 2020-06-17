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
    User(user::Model),
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
            Route::User(id) => Self::user(id),
            Route::NotFound => Self::not_found(),
        }
    }
}
impl Model {
    pub fn home() -> Self {
        Self::Home(home::Model::default())
    }
    pub fn users() -> Self {
        Self::Users(users::Model::fetch_all())
    }
    pub fn user(id: Id<User>) -> Self {
        Self::User(user::Model::from(id))
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
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    User(user::Msg),
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
impl From<user::Msg> for Msg {
    fn from(msg: user::Msg) -> Self {
        Self::User(msg)
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
    match msg {
        Msg::Home(msg) => {
            if let Model::Home(home) = model {
                home::update(
                    msg,
                    home,
                    &mut orders.proxy(Msg::Home)
                );
            }
        },
        Msg::Login(msg) => {
            if let Model::Login(login) = model {
                login::update(
                    msg,
                    login,
                    &mut orders.proxy(Msg::Login)
                );
            }
        },
        Msg::Register(msg) => {
            if let Model::Register(register) = model {
                register::update(
                    msg,
                    register,
                    &mut orders.proxy(Msg::Register)
                );
            }
        },
        Msg::Users(msg) => {
            if let Model::Users(users) = model {
                users::update(
                    msg,
                    users,
                    &mut orders.proxy(Msg::Users)
                );
            }
        },
        Msg::User(msg) => {
            if let Model::User(user) = model {
                user::update(
                    msg,
                    user,
                    &mut orders.proxy(Msg::User)
                );
            }
        },
        Msg::FetchData => {
            match model {
                Model::User(user) => {
                    user::update(
                        user::Msg::FetchUser,
                        user,
                        &mut orders.proxy(Msg::User)
                    );
                },
                Model::Users(users) => {
                    users::update(
                        users::Msg::FetchUsers,
                        users,
                        &mut orders.proxy(Msg::Users)
                    );
                },
                _ => {},
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::Home(home) =>
            home::view(&home)
                .map_msg(Msg::Home),
        Model::User(user) =>
            user::view(&user)
                .map_msg(Msg::User),
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
