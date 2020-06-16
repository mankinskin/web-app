use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use crate::{
    root::{
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
}

pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::SetSession(session) => {
            model.session = Some(session);
        },
        Msg::EndSession => {
            model.session = None;
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
        if let Some(session) = &model.session {
            div![
                a![
                    attrs!{
                        At::Href => format!("/users/{}", session.user_id);
                    },
                    "My Profile",

                ],
            ]
        } else { empty![] },
    ]
}
