use seed::{
    *,
    prelude::*,
    browser::fetch::FetchError,
};
use app_model::{
    credentials::*,
    user::*,
};
use crate::{
    Component,
    View,
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
    LoginSuccess(UserSession),
    Register,
}

pub async fn login_request(credentials: Credentials) -> Result<UserSession, FetchError> {
    let url = format!("{}{}", "http://localhost:8000", "/api/auth/login");
    let req = seed::fetch::Request::new(&url)
        .method(Method::Post);
    seed::fetch::fetch(
        req.json(&credentials)?
    )
    .await?
    .check_status()?
    .json()
    .await
    .map(|session: UserSession| session)
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
                    login_request(self.credentials.clone())
                        .map(|result: Result<UserSession, FetchError>| {
                            Msg::LoginResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::LoginResponse(result) => {
                match result {
                    Ok(session) => {
                        orders.notify(Msg::LoginSuccess(session));
                    },
                    Err(e) => {seed::log!(e)}
                }
            },
            Msg::LoginSuccess(_session) => {},
            Msg::Register => {},
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
