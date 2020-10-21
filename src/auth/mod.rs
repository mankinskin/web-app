#[cfg(not(target_arch = "wasm32"))]
pub mod jwt;

pub mod credentials;
#[cfg(target_arch = "wasm32")]
pub mod login;
#[cfg(target_arch = "wasm32")]
pub use login::*;
#[cfg(target_arch = "wasm32")]
pub mod register;
#[cfg(target_arch = "wasm32")]
pub use register::*;
#[cfg(target_arch = "wasm32")]
pub mod session;
use crate::user::*;
#[cfg(target_arch = "wasm32")]
use components::{
    Component,
    Init,
    Viewable,
};
use rql::Id;
#[cfg(target_arch = "wasm32")]
use seed::prelude::*;
#[cfg(target_arch = "wasm32")]
pub use session::Session;
#[cfg(target_arch = "wasm32")]
use tracing::debug;

#[cfg(not(target_arch = "wasm32"))]
pub use {
    credentials::*,
    database_table::{
        Database,
        DatabaseTable,
    },
    jwt::*,
    std::convert::TryFrom,
    actix_web::error::{
        Error,
        ErrorUnauthorized,
        ErrorNotFound,
        ErrorInternalServerError,
        ErrorConflict,
    },
};
use serde::{
    Serialize,
    Deserialize,
};
use enum_paths::{
    AsPath,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Id<User>,
    pub token: String,
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn login<'db, D: Database<'db, User>>(credentials: Credentials) -> Result<UserSession, Error> {
    DatabaseTable::<'db, D>::find(|user| *user.name() == credentials.username)
        .ok_or(ErrorNotFound("User not found"))
        .and_then(|entry| {
            let user = entry.data();
            if *user.password() == credentials.password {
                Ok(entry)
            } else {
                Err(ErrorUnauthorized("Unauthorized"))
            }
        })
        .and_then(|entry| {
            let user = entry.data().clone();
            let id = entry.id().clone();
            JWT::try_from(&user)
                .map_err(|_| ErrorInternalServerError(""))
                .map(move |jwt| (id, jwt))
        })
        .map(|(id, jwt)| {
            UserSession {
                user_id: id.clone(),
                token: jwt.to_string(),
            }
        })
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn register<'db, D: Database<'db, User>>(user: User) -> Result<UserSession, Error> {
    if DatabaseTable::<'db, D>::find(|u| u.name() == user.name()).is_none() {
        let id = DatabaseTable::<'db, D>::insert(user.clone());
        JWT::try_from(&user)
            .map_err(|_| ErrorInternalServerError(""))
            .map(move |jwt| {
                UserSession {
                    user_id: id.clone(),
                    token: jwt.to_string(),
                }
            })
    } else {
        Err(ErrorConflict("Username already taken"))
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub enum Auth {
    Login(Login),
    Register(Register),
    Session(Session),
}
#[cfg(target_arch = "wasm32")]
impl Auth {
    pub fn login() -> Self {
        Auth::Login(Login::default())
    }
    pub fn register() -> Self {
        Auth::Register(Register::default())
    }
    pub fn session(session: Session) -> Self {
        Auth::Session(session)
    }
}

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Login,
    Register,
}
impl database_table::Route for Route {}

#[cfg(target_arch = "wasm32")]
impl Init<()> for Auth {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(Msg::Set);
        Self::login()
    }
}
#[cfg(target_arch = "wasm32")]
impl From<Route> for Auth {
    fn from(route: Route) -> Self {
        match route {
            Route::Login => Self::login(),
            Route::Register => Self::register(),
        }
    }
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub enum Msg {
    Set(Auth),
    Login(login::Msg),
    Register(register::Msg),
    Session(session::Msg),
}
#[cfg(target_arch = "wasm32")]
impl Component for Auth {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        if let Msg::Set(auth) = msg {
            debug!("Setting Auth");
            *self = auth;
        } else if let Auth::Login(login) = self {
            if let Msg::Login(msg) = msg {
                login.update(msg, &mut orders.proxy(Msg::Login));
            }
        } else if let Auth::Register(register) = self {
            if let Msg::Register(msg) = msg {
                register.update(msg, &mut orders.proxy(Msg::Register));
            }
        } else if let Auth::Session(session) = self {
            if let Msg::Session(msg) = msg {
                session.update(msg, &mut orders.proxy(Msg::Session));
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Viewable for Auth {
    fn view(&self) -> Node<Msg> {
        match self {
            Auth::Login(login) => login.view().map_msg(Msg::Login),
            Auth::Register(register) => register.view().map_msg(Msg::Register),
            Auth::Session(session) => session.view().map_msg(Msg::Session),
        }
    }
}
