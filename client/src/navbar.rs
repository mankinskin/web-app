use crate::{
    Route,
    page::Page,
};
use app_model::{
    auth::{
        self,
        UserSession,
    },
    user,
    project,
};
use components::{
    Component,
    Viewable,
    Init,
    router::{
        self,
        Router,
        ToRoute,
    },
};
use seed::{
    prelude::*,
    *,
};
use enum_paths::AsPath;

#[derive(Debug)]
pub struct Navbar {
    router: Router<Route, Page>
}

impl<T: ToRoute<Route>> Init<T> for Navbar {
    fn init(t: T, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            router: router::Router::init(t.to_route(), &mut orders.proxy(Msg::Router)),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Router(router::Msg<Route, Page>),
    SetSession(UserSession),
    EndSession,
}
impl Component for Navbar {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Navbar Update");
        match msg {
            Msg::Router(msg) => self.router.update(msg, &mut orders.proxy(Msg::Router)),
            Msg::SetSession(session) => {
                auth::session::set(session);
            }
            Msg::EndSession => {
                auth::session::end();
                self.router.set_page(Page::default());
            }
        }
    }
}
impl Viewable for Navbar {
    fn view(&self) -> Node<Msg> {
        div![
            div![a![
                attrs! {
                    At::Href => Route::Root.as_path();
                },
                "Home",
            ],],
            if let Some(session) = auth::session::get() {
                div![
                    a![
                        attrs! {
                            At::Href => Route::User(user::Route::Users).as_path();
                        },
                        "Users",
                    ],
                    a![
                        attrs! {
                            At::Href => Route::Project(project::Route::Projects).as_path();
                        },
                        "Projects",
                    ],
                    a![
                        attrs! {
                            At::Href => Route::User(user::Route::User(session.user_id)).as_path();
                        },
                        "My Profile",
                    ],
                    button![ev(Ev::Click, |_| Msg::EndSession), "Log Out"],
                ]
            } else {
                div![
                    div![a![
                        attrs! {
                            At::Href => Route::Auth(auth::Route::Login).as_path();
                        },
                        "Login",
                    ],],
                    div![a![
                        attrs! {
                            At::Href => Route::Auth(auth::Route::Register).as_path();
                        },
                        "Register",
                    ],],
                ]
            },
            self.router.view().map_msg(Msg::Router),
        ]
    }
}
