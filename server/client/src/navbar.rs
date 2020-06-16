use seed::{
    *,
    prelude::*,
};

#[derive(Clone, Default)]
pub struct Model {
}

#[derive(Clone)]
pub enum Msg {
}

pub fn update(_msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg>) {
}
pub fn view(_model: &Model) -> Node<Msg> {
    div![
        div![
            a![
                attrs!{
                    At::Href => "/";
                },
                "Home",
            ],
        ],
        div![
            a![
                attrs!{
                    At::Href => "/login";
                },
                "Login",
            ],
        ],
        div![
            a![
                attrs!{
                    At::Href => "/register";
                },
                "Register",

            ],
        ],
        div![
            a![
                attrs!{
                    At::Href => "/users";
                },
                "Users",

            ],
        ],
    ]
}
