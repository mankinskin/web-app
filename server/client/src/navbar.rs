use seed::{
    *,
    prelude::*,
};
use crate::{
    root::{
        self,
        GMsg,
    },
};

#[derive(Clone)]
pub struct Model {
}

impl Default for Model {
    fn default() -> Self {
        Self {
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Logout,
}

pub fn update(msg: Msg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Logout => {
            orders.send_g_msg(root::GMsg::EndSession);
        },
    }
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
        if let Some(session) = api::auth::get_session() {
            div![
                a![
                    attrs!{
                        At::Href => format!("/users");
                    },
                    "Users",
                ],
                a![
                    attrs!{
                        At::Href => format!("/projects");
                    },
                    "Projects",
                ],
                a![
                    attrs!{
                        At::Href => format!("/users/{}", session.user_id);
                    },
                    "My Profile",
                ],
                button![simple_ev(Ev::Click, Msg::Logout), "Log Out"],
            ]
        } else {
            div![
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
            ]
        },
    ]
}
