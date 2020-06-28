use plans::{
    user::*,
};
use seed::{
    *,
    prelude::*,
};
use crate::{
    page,
    login,
    route::{
        Route,
    },
    root::{
        self,
        GMsg,
    },
};

#[derive(Clone, Default)]
pub struct Model {
    user: User,
}
impl Model {
    pub fn empty() -> Self {
        Self::default()
    }
}
#[derive(Clone, Default)]
pub struct Config {
}
pub fn init(_config: Config, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
    Model::default()
}
#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    Submit,
    RegistrationResponse(Result<UserSession, String>),
    Login,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeUsername(u) => {
            model.user.credentials_mut().username = u;
        },
        Msg::ChangePassword(p) => {
            model.user.credentials_mut().password = p;
        },
        Msg::Submit => {
            seed::log!("Registration...");
            orders.perform_cmd(
                api::register(model.user.clone())
                    .map(|result: Result<UserSession, FetchError>| {
                        Msg::RegistrationResponse(result.map_err(|e| format!("{:?}", e)))
                    })
            );
        },
        Msg::RegistrationResponse(result) => {
            match result {
                Ok(session) => {
                    seed::log!("Ok");
                    orders.send_g_msg(root::GMsg::SetSession(session));
                    page::go_to(Route::Home, orders);
                },
                Err(e) => {seed::log!(e)}
            }
        },
        Msg::Login => {
            page::go_to(login::Config::default(), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    // registration form
    form![
        // Username field
        label![
            "Username"
        ],
        input![
            attrs!{
                At::Placeholder => "Username",
                At::Value => model.user.credentials().username,
            },
            input_ev(Ev::Input, Msg::ChangeUsername)
        ],
        div![
            model.user.credentials().username_invalid_text()
        ],
        // Password field
        label![
            "Password"
        ],
        input![
            attrs!{
                At::Type => "password",
                At::Placeholder => "Password",
                At::Value => model.user.credentials().password,
            },
            input_ev(Ev::Input, Msg::ChangePassword)
        ],
        div![
            model.user.credentials().password_invalid_text()
        ],
        // Submit Button
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
        // Login Button
        button![simple_ev(Ev::Click, Msg::Login), "Log In"],
    ]
}
