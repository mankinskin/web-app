use plans::{
    user::*,
};
use budget::{
    currency::*,
};
use crate::{
    budget_view::*,
};
use yew::{
    *,
};
use common::{
    fetch::*,
};
use url::{
    *,
};
use std::fmt::{Debug};

pub enum Msg {
    User(FetchResponse<User>),
}
impl From<FetchResponse<User>> for Msg {
    fn from(m: FetchResponse<User>) -> Self {
        Self::User(m)
    }
}

#[derive(Properties, Clone, Debug)]
pub struct UserProfileData {
    pub user: Url,
}
pub struct UserProfileView {
    props: UserProfileData,
    link: ComponentLink<Self>,
    user: Option<User>,
}

impl Component for UserProfileView {
    type Message = Msg;
    type Properties = UserProfileData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let s = Self {
            props,
            link,
            user: None,
        };
        Fetch::get(s.props.user.clone())
            .responder(s.link.callback(|response| {
                Msg::User(response)
            }))
            .send()
            .expect("Fetch request failed");
        s
    }
    fn view(&self) -> Html {
        console!(log, "Draw UserProfileView");
        if let Some(user) = self.user.clone() {
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
            Msg::User(res) => {
                match res.into_inner() {
                    Ok(user) => self.user = Some(user),
                    Err(e) => console!(log, format!("{:#?}", e)),
                }
            },
        }
        true
    }
}
