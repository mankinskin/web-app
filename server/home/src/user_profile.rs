use plans::{
    *,
    user::*,
};
use budget::{
    *,
    currency::*,
};
use crate::{
    budget_view::*,
};
use yew::{
    *,
};
use common::{
    remote_data::*,
    database::*,
};
use rql::{
    *,
};
use std::result::Result;
use std::fmt::{Debug};
use futures::{Future, FutureExt};

pub enum Msg {
    RemoteUser(RemoteMsg<User>),
}
impl From<RemoteMsg<User>> for Msg {
    fn from(m: RemoteMsg<User>) -> Self {
        Self::RemoteUser(m)
    }
}

#[derive(Properties, Clone, Debug)]
pub struct UserProfileData {
    pub user: RemoteRoute,
}
pub struct UserProfileView {
    props: UserProfileData,
    link: ComponentLink<Self>,
    user: RemoteData<User, Self>,
}

impl Component for UserProfileView {
    type Message = Msg;
    type Properties = UserProfileData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let user = RemoteData::new(props.user.clone(), link.clone());
        let s = Self {
            props,
            link,
            user,
        };
        s
    }
    fn view(&self) -> Html {
        console!(log, "Draw UserProfileView");
        if let Some(user) = self.user.data().clone() {
            html!{
                <div id="user-profile">
                    <div id="user-profile-header" class="profile-card">
                        <div class="user-profile-image-container">
                            <img class="user-profile-image" src="/img/dweeb.jpg"/>
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
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RemoteUser(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            self.user.fetch_request(request)
                                .expect("Failed to make request")
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.user.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
        }
        true
    }
}
