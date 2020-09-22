use crate::{
    Component,
    View,
    auth::{
        self,
        Auth,
        login::Login,
        session::Session,
    },
};
use app_model::{
    user::*,
    auth::{
        UserSession,
    },
};
use seed::{prelude::*, *};

#[derive(Debug, Clone, Default)]
pub struct Register {
    user: User,
}
#[derive(Clone, Debug)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    Submit,
    RegistrationResponse(Result<UserSession, String>),
    Login,
}
pub async fn registration_request(user: User) -> Result<UserSession, FetchError> {
    let url = format!("http://localhost:8000/api/register");
    let req = seed::fetch::Request::new(&url).method(Method::Post);
    seed::fetch::fetch(req.json(&user)?)
        .await?
        .check_status()?
        .json()
        .await
        .map(|session: UserSession| session)
}
impl Component for Register {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::ChangeUsername(u) => {
                self.user.credentials_mut().username = u;
            },
            Msg::ChangePassword(p) => {
                self.user.credentials_mut().password = p;
            },
            Msg::Submit => {
                seed::log!("Registration...");
                orders.perform_cmd(registration_request(self.user.clone()).map(
                    |result: Result<UserSession, FetchError>| {
                        Msg::RegistrationResponse(result.map_err(|e| format!("{:?}", e)))
                    },
                ));
            },
            Msg::RegistrationResponse(result) => {
                seed::log!("Registration Success");
                match result {
                    Ok(session) => {
                        seed::log!("Ok");
                        orders.notify(Auth::Session(Session::from(session)));
                    }
                    Err(e) => seed::log!(e),
                }
            },
            Msg::Login => {
                orders.notify(auth::Msg::Set(Auth::Login(Login::default())));
            }
        }
    }
}
impl View for Register {
    fn view(&self) -> Node<Msg> {
        form![
            label!["Username"],
            input![
                attrs! {
                    At::Placeholder => "Username",
                    At::Value => self.user.credentials().username,
                },
                input_ev(Ev::Input, Msg::ChangeUsername)
            ],
            div![self.user.credentials().username_invalid_text()],
            label!["Password"],
            input![
                attrs! {
                    At::Type => "password",
                    At::Placeholder => "Password",
                    At::Value => self.user.credentials().password,
                },
                input_ev(Ev::Input, Msg::ChangePassword)
            ],
            div![self.user.credentials().password_invalid_text()],
            button![
                attrs! {
                    At::Type => "submit",
                },
                "Register"
            ],
            ev(Ev::Submit, |ev| {
                ev.prevent_default();
                Msg::Submit
            }),
            button![ev(Ev::Click, |_| Msg::Login), "Log In"],
        ]
    }
}
