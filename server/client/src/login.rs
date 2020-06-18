use seed::{
    *,
    prelude::*,
    browser::service::fetch::{
        FailReason,
    },
};
use plans::{
    credentials::*,
    user::*,
};
use crate::{
    request,
    page,
    register,
    route::{
        self,
        Route,
    },
    root::{
        self,
        GMsg,
    },
};
/// credential input component
#[derive(Clone, Default)]
pub struct Model {
    credentials: Credentials,
    submit_result: Option<Result<(), FailReason<UserSession>>>,
}
impl Model {
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    LoginResponse(ResponseDataResult<UserSession>),
    Login,
    Register,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeUsername(u) => model.credentials.username = u,
        Msg::ChangePassword(p) => model.credentials.password = p,
        Msg::Login => {
            seed::log!("Logging in...");
            orders.perform_cmd(request::login_request(model.credentials()));
        },
        Msg::LoginResponse(res) => {
            model.submit_result = Some(res.clone().map(|_| ()));
            match res {
                Ok(session) => {
                    orders.send_g_msg(root::GMsg::SetSession(session));
                    route::change_route(Route::Home, orders);
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::Register => {
            page::go_to(register::Model::default(), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    // login form
    form![
        // Username field
        label![
            "Username"
        ],
        input![
            attrs!{
                At::Placeholder => "Username",
                At::Value => model.credentials.username,
            },
            input_ev(Ev::Input, Msg::ChangeUsername)
        ],
        div![
            model.credentials.username_invalid_text()
        ],
        // Password field
        label![
            "Password"
        ],
        input![
            attrs!{
                At::Type => "password",
                At::Placeholder => "Password",
                At::Value => model.credentials.password,
            },
            input_ev(Ev::Input, Msg::ChangePassword)
        ],
        div![
            model.credentials.password_invalid_text()
        ],
        // Login Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Login"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Login
        }),
        // Register Button
        button![simple_ev(Ev::Click, Msg::Register), "Register"],
        if let Some(res) = &model.submit_result {
            p![format!("{:?}", res)]
        } else { empty![] }
    ]
}
