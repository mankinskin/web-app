use crate::{
    Component,
    Viewable,
    auth::{
        Auth,
        register::Register,
        session::Session,
    },
};
use app_model::{
    auth::{
        credentials::Credentials,
        UserSession,
    },
};
use seed::{browser::fetch::FetchError, prelude::*, *};
use std::result::Result;

#[derive(Debug, Clone)]
pub struct Login {
    pub url: String,
    pub credentials: Credentials,
}
#[derive(Debug, Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    LoginResponse(Result<UserSession, String>),
    Submit,
    Register,
}
impl Login {
    async fn login_request(self) -> Result<UserSession, FetchError> {
        let req = seed::fetch::Request::new(&self.url).method(Method::Post);
        seed::fetch::fetch(req.json(&self.credentials)?)
            .await?
            .check_status()?
            .json()
            .await
            .map(|session: UserSession| session)
    }
}
impl Default for Login {
    fn default() -> Self {
        Self {
            credentials: Default::default(),
            url: "http://localhost:8000/api/login".into(),
        }
    }
}
impl Component for Login {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::ChangeUsername(u) => self.credentials.username = u,
            Msg::ChangePassword(p) => self.credentials.password = p,
            Msg::Submit => {
                seed::log!("Logging in...");
                orders.perform_cmd(self.clone().login_request().map(
                    |result: Result<UserSession, FetchError>| {
                        Msg::LoginResponse(result.map_err(|e| format!("{:?}", e)))
                    },
                ));
            }
            Msg::LoginResponse(result) => {
                seed::log!("Login Response");
                match result {
                    Ok(session) => {
                        orders.notify(Auth::Session(Session::from(session)));
                    }
                    Err(e) => seed::log!(e),
                }
            },
            Msg::Register => {
                orders.notify(Auth::Register(Register::default()));
            }
        }
    }
}
impl Viewable for Login {
    fn view(&self) -> Node<Msg> {
        //seed::log!("Login redraw");
        form![
            label!["Username"],
            input![
                attrs! {
                    At::Placeholder => "Username",
                    At::Value => self.credentials.username,
                },
                input_ev(Ev::Input, Msg::ChangeUsername)
            ],
            div![self.credentials.username_invalid_text()],
            label!["Password"],
            input![
                attrs! {
                    At::Type => "password",
                    At::Placeholder => "Password",
                    At::Value => self.credentials.password,
                },
                input_ev(Ev::Input, Msg::ChangePassword)
            ],
            div![self.credentials.password_invalid_text()],
            button![
                attrs! {
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
