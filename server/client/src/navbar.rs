use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use crate::{
    root::{
        self,
        GMsg,
    },
};

#[derive(Clone)]
pub struct Model {
    session: Option<UserSession>,
}

impl Model {
    pub fn with_session(session: UserSession) -> Self {
        Self {
            session: Some(session),
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self {
            session: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    SetSession(UserSession),
    EndSession,
    Logout,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::SetSession(session) => {
            model.session = Some(session);
        },
        Msg::EndSession => {
            model.session = None;
        },
        Msg::Logout => {
            orders.send_g_msg(GMsg::Root(root::Msg::EndSession));
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        div![
            a![
                attrs!{
                    At::Href => "/";
                },
                "Home",
            ],
        ],
        if let Some(session) = &model.session {
            div![
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
