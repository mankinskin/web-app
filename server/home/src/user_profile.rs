pub use plans::{
    *,
    user::*,
};
use yew::{
    *,
};
use crate::{
    *,
    budget::*,
};
use common::{
    remote_data::*,
};
use rql::{
    *,
};
use anyhow::Error;
use std::result::Result;
use std::fmt::{Debug};
use futures::{Future, FutureExt};

pub enum Msg {
    RemoteUser(RemoteMsg<User>),
}

#[derive(Properties, Clone, Debug)]
pub struct UserProfileData {
    pub user: RemoteData<User>,
}
pub struct UserProfileView {
    props: UserProfileData,
    link: ComponentLink<Self>,
}
impl UserProfileView {
    fn user_responder(&self) -> Callback<RemoteResponse<User>> {
        self.link.callback(move |response: RemoteResponse<User>| {
            Msg::RemoteUser(RemoteMsg::Response(response))
        })
    }
    fn user_request(&self, request: RemoteRequest<User>) -> impl Future<Output=()> + 'static {
        let callback = self.user_responder().clone();
        self.props.user.fetch_request(request)
            .then(move |res: RemoteResponse<User>| {
                futures::future::ready(callback.emit(res))
            })
    }
}
impl Component for UserProfileView {
    type Message = Msg;
    type Properties = UserProfileData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Self {
            props,
            link,
        };
        s.link.send_message(Msg::RemoteUser(RemoteMsg::Request(RemoteRequest::Get(Id::new()))));
        s
    }
    fn view(&self) -> Html {
        console!(log, "Draw UserProfileView");
        if let Some(user) = self.props.user.data().clone() {
            html!{
                <div id="user-profile">
                    <div id="user-profile-header" class="profile-card">
                        <div class="user-profile-image-container">
                            <img class="user-profile-image" src="/dweeb.jpg"/>
                        </div>
                        <div id="user-info-container">
                            <div id="user-name">
                                {format!("{}", user.name())}
                            </div>
                        </div>
                    </div>
                    <div id="user-profile-posts" class="profile-card">
                    </div>
                    <div id="user-profile-budget" class="profile-card">
                        <BudgetView<Euro> />
                    </div>
                </div>
            }
        } else {
            html!{ }
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RemoteUser(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            self.user_request(request)
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.props.user.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
        }
        true
    }
}
