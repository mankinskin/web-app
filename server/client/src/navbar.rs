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

#[derive(Debug,Clone, Default)]
pub struct Model {
}
#[derive(Clone, Debug)]
pub enum Msg {
    Logout,
}

impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Logout => {
                orders.notify(root::Msg::EndSession);
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
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
                    button![ev(Ev::Click, |_| Msg::Logout), "Log Out"],
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
}
