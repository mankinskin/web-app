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
    request,
    route::{
        self,
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
    submit_result: Option<ResponseDataResult<UserSession>>,
}
#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    Submit,
    RegistrationResponse(ResponseDataResult<UserSession>),
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
            orders.perform_cmd(request::registration_request(&model.user));
        },
        Msg::RegistrationResponse(res) => {
            model.submit_result = Some(res.clone());
            match res {
                Ok(session) => {
                    seed::log!("Ok");
                    orders.send_g_msg(root::GMsg::SetSession(session));
                    route::change_route(Route::Home, orders);
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::Login => {
            page::go_to(login::Model::default(), orders);
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
        if let Some(res) = &model.submit_result {
            p![format!("{:?}", res)]
        } else { empty![] }
    ]
}
