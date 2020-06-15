use plans::{
    user::*,
};
use seed::{
    *,
    prelude::*,
};
use futures::{
    Future,
};

#[derive(Default)]
pub struct Model {
    user: User,
}

#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    Submit,
    RegistrationResponse(ResponseDataResult<()>),
    Login,
}

fn registration_request(user: &User)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/users/register")
        .method(Method::Post)
        .send_json(user)
        .fetch_json_data(move |data_result: ResponseDataResult<()>| {
            Msg::RegistrationResponse(data_result)
        })
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangeUsername(u) => {
            model.user.credentials_mut().username = u;
        },
        Msg::ChangePassword(p) => {
            model.user.credentials_mut().password = p;
        },
        Msg::Submit => {
            seed::log!("Registration...");
            orders.perform_cmd(registration_request(&model.user));
        },
        Msg::RegistrationResponse(res) => {
            match res {
                Ok(()) => {
                    seed::log!("Ok");
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::Login => {
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
