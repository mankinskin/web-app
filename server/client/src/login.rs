use seed::{
    *,
    prelude::*,
    browser::fetch::FetchError,
};
use plans::{
    credentials::*,
    user::*,
};
use crate::{
    root,
    components::{
        Component,
        View,
    },
};
use api::{
    routes::{
        Route,
    },
};
use std::result::Result;

#[derive(Debug,Clone, Default)]
pub struct Model {
    pub credentials: Credentials,
}
#[derive(Debug,Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    LoginResponse(Result<UserSession, String>),
    Submit,
    Register,
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::ChangeUsername(u) => self.credentials.username = u,
            Msg::ChangePassword(p) => self.credentials.password = p,
            Msg::Submit => {
                seed::log!("Logging in...");
                orders.perform_cmd(
                    api::login(self.credentials.clone())
                        .map(|result: Result<UserSession, FetchError>| {
                            Msg::LoginResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::LoginResponse(result) => {
                match result {
                    Ok(session) => {
                        orders.notify(root::Msg::SetSession(session));
                        root::go_to(Route::Home, orders);
                    },
                    Err(e) => {seed::log!(e)}
                }
            },
            Msg::Register => {
                root::go_to(Route::Register, orders);
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        form![
            label![
                "Username"
            ],
            input![
                attrs!{
                    At::Placeholder => "Username",
                    At::Value => self.credentials.username,
                },
                input_ev(Ev::Input, Msg::ChangeUsername)
            ],
            div![
                self.credentials.username_invalid_text()
            ],
            label![
                "Password"
            ],
            input![
                attrs!{
                    At::Type => "password",
                    At::Placeholder => "Password",
                    At::Value => self.credentials.password,
                },
                input_ev(Ev::Input, Msg::ChangePassword)
            ],
            div![
                self.credentials.password_invalid_text()
            ],
            button![
                attrs!{
                    At::Type => "submit",
                },
                "Login"
            ],
            ev(Ev::Submit, |ev| {
                ev.prevent_default();
                Msg::Submit
            }),
            button![ev(Ev::Click, |_| Msg::Register), "Register"],
        ]
    }
}
