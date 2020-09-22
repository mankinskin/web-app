pub mod login;
pub mod register;
pub mod session;

use seed::prelude::*;
use login::Login;
use register::Register;
use session::Session;
use crate::{
    View,
    Component,
    Init,
};

#[derive(Debug, Clone)]
pub enum Auth {
    Login(Login),
    Register(Register),
    Session(Session),
}

impl Init<()> for Auth {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(Msg::Set);
        Auth::Login(Login::default())
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
            *self = auth;
        }
        else if let Auth::Login(login) = self {
            if let Msg::Login(msg) = msg {
                login.update(msg, &mut orders.proxy(Msg::Login));
            }
        }
        else if let Auth::Register(register) = self {
            if let Msg::Register(msg) = msg {
                register.update(msg, &mut orders.proxy(Msg::Register));
            }
        }
        else if let Auth::Session(session) = self {
            if let Msg::Session(msg) = msg {
                session.update(msg, &mut orders.proxy(Msg::Session));
            }
        }
    }
}
impl View for Auth {
    fn view(&self) -> Node<Msg> {
        match self {
            Auth::Login(login) => login.view().map_msg(Msg::Login),
            Auth::Register(register) => register.view().map_msg(Msg::Register),
            Auth::Session(session) => session.view().map_msg(Msg::Session),
        }
    }
}
