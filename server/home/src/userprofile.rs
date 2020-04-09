pub use plans::{
    *,
    user::*,
};
use yew::{
    *,
    services::{
        *,
        fetch::{
            *,
            FetchTask,
        },
    },
    format::{
        Json,
        Nothing,
    },
};
use http::{
    *,
    request::*,
};
use crate::{
    *,
};
use anyhow::Error;
use std::result::Result;
use std::fmt::{Display, Debug};

pub enum Msg {
    GotUser(User),
    FetchUserError(String),
}

#[derive(Properties, Clone, Debug)]
pub struct UserProfile {
    user: User,
}
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            user,
        }
    }
}
pub struct UserProfileView {
    link: ComponentLink<Self>,
    props: Option<UserProfile>,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
}

impl Component for UserProfileView {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props: None,
            fetch_service: FetchService::new(),
            fetch_task: None,
        }
    }
    fn mounted(&mut self) -> ShouldRender {
        let req = Request::get("/api/user")
            .body(Nothing).unwrap();
        let callback = self.link.callback(|response: Response<Json<Result<User, Error>>>| {
            console!(log, "Got response");
            let (meta, Json(body)) = response.into_parts();
            if meta.status.is_success() {
                console!(log, "Got user");
                Msg::GotUser(body.unwrap())
            } else {
                Msg::FetchUserError(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status))
                )
            }
        });
        let task = self.fetch_service.fetch(req, callback);
        match task {
            Ok(task) => {
                self.fetch_task = Some(task)
            },
            Err(err) => {
                self.link.send_message(Msg::FetchUserError(err.to_string()))
            },
        }
        false
    }
    fn view(&self) -> Html {
        if let Some(profile) = self.props.clone() {
            html!{
                <div class="user-profile">
                    <p class="user-name">
                        {format!("User Name: {}", profile.user.name())}
                    </p>
                </div>
            }
        } else {
            html!{
                <div>
                    {format!("Failed to get user!")}
                </div>
            }
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotUser(user) => {
                self.props = Some(UserProfile::from(user));
                true
            }
            Msg::FetchUserError(err) => {
                console!(log, "FetchUserError: {}", format!("{}", err));
                true
            },
        }
    }
}
