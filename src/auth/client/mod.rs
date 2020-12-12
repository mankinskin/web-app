pub mod login;
pub use login::*;
pub mod register;
pub use register::*;
pub mod session;
pub use session::Session;

use crate::{
    auth::Route,
};
use components::{
    Component,
    Init,
    Viewable,
};
use seed::prelude::*;
use tracing::debug;

pub fn get_location() -> web_sys::Location {
    web_sys::window().unwrap().location()
}
pub fn get_host() -> Result<String, JsValue> {
    get_location().host()
}
pub fn get_base_url() -> Result<String, JsValue> {
    let loc = get_location();
    Ok(format!("{}://{}:{}",
        loc.protocol()?,
        loc.host()?,
        loc.port()?,
    ))
}

#[derive(Debug, Clone)]
pub enum Auth {
    Login(Login),
    Register(Register),
    Session(Session),
}
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
impl Init<()> for Auth {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(Msg::Set);
        Self::login()
    }
}
impl From<Route> for Auth {
    fn from(route: Route) -> Self {
        match route {
            Route::Login => Self::login(),
            Route::Register => Self::register(),
        }
    }
}
impl Init<Route> for Auth {
    fn init(route: Route, _orders: &mut impl Orders<Msg>) -> Self {
        Self::from(route)
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Set(Auth),
    Login(login::Msg),
    Register(register::Msg),
    Session(session::Msg),
}
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
impl Viewable for Auth {
    fn view(&self) -> Node<Msg> {
        match self {
            Auth::Login(login) => login.view().map_msg(Msg::Login),
            Auth::Register(register) => register.view().map_msg(Msg::Register),
            Auth::Session(session) => session.view().map_msg(Msg::Session),
        }
    }
}
