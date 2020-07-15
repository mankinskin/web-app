use plans::{
    user::*,
};
use seed::{
    *,
    prelude::*,
};
use crate::{
    root,
    config::{
        Component,
        View,
    },
};
use api::{
    routes::{
        Route,
    },
};

#[derive(Debug,Clone, Default)]
pub struct Model {
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
impl Component for Model {
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
                orders.perform_cmd(
                    api::register(self.user.clone())
                        .map(|result: Result<UserSession, FetchError>| {
                            Msg::RegistrationResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::RegistrationResponse(result) => {
                match result {
                    Ok(session) => {
                        seed::log!("Ok");
                        orders.notify(root::Msg::SetSession(session));
                        root::go_to(Route::Home, orders);
                    },
                    Err(e) => {seed::log!(e)}
                }
            },
            Msg::Login => {
                root::go_to(Route::Login, orders);
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
                    At::Value => self.user.credentials().username,
                },
                input_ev(Ev::Input, Msg::ChangeUsername)
            ],
            div![
                self.user.credentials().username_invalid_text()
            ],
            label![
                "Password"
            ],
            input![
                attrs!{
                    At::Type => "password",
                    At::Placeholder => "Password",
                    At::Value => self.user.credentials().password,
                },
                input_ev(Ev::Input, Msg::ChangePassword)
            ],
            div![
                self.user.credentials().password_invalid_text()
            ],
            button![
                attrs!{
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
