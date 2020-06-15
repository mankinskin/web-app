use seed::{
    *,
    prelude::*,
};
use futures::{
    Future,
};
use plans::{
    credentials::*,
    user::*,
};
/// credential input component
#[derive(Clone, Default)]
pub struct Model {
    credentials: Credentials,
    session: Option<UserSession>,
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
    Login,
    LoginResponse(ResponseDataResult<UserSession>),
    Register,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangeUsername(u) => model.credentials.username = u,
        Msg::ChangePassword(p) => model.credentials.password = p,
        Msg::Login => {
            seed::log!("Logging in...");
            orders.perform_cmd(login_request(model.credentials()));
        },
        Msg::LoginResponse(res) => {
            match res {
                Ok(session) => {
                    seed::log!(session);
                    model.session = Some(session);
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::Register => {},
    }
}
fn login_request(credentials: &Credentials)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/users/login")
        .method(Method::Post)
        .send_json(credentials)
        .fetch_json_data(move |data_result: ResponseDataResult<UserSession>| {
            Msg::LoginResponse(data_result)
        })
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
    ]
}
