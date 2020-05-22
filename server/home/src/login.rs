use yew::{
    *,
};
pub use plans::{
    *,
    user::*,
    credentials::*,
};
use crate::{
    *,
};
use common::{
    remote_data::*,
};
use updatable::{
    *,
};
use stdweb::web::{
    *,
    html_element::{InputElement},
};
use stdweb::unstable::TryInto;

#[derive(Properties, Clone, Debug)]
pub struct LoginData {
    pub login: RemoteRoute,
    pub credentials: Option<Credentials>,
}
pub struct Login {
    link: ComponentLink<Self>,
    props: LoginData,
}
pub enum Msg {
    UpdateCredentials(CredentialsUpdate),
    Login,
    ToggleShowPassword,
}

impl Login {
    fn set_username_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::UpdateCredentials(
                Credentials::update()
                    .username(input.value)
            )
        })
    }
    fn set_password_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::UpdateCredentials(
                Credentials::update()
                .password(input.value)
            )
        })
    }
    fn login_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::Login
        })
    }
    fn toggle_show_password_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(|click_event: ClickEvent| {
            let password_input: InputElement = stdweb::web::document()
                .query_selector("#password-input").unwrap()
                .expect("password-input not found")
                .try_into()
                .expect("Failed to cast to InputElement");
            password_input.set_attribute("type",
                match password_input.get_attribute("type") {
                    None => {
                        "password"
                    },
                    Some(s) => {
                        if s.contains("password") {
                            ""
                        } else {
                            "password"
                        }
                    }
                }
            );
            Msg::ToggleShowPassword
        })
    }
}
impl Component for Login {
    type Message = Msg;
    type Properties = LoginData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let s = Self {
            props,
            link,
        };
        s
    }
    fn view(&self) -> Html {
        console!(log, "Draw UserProfileView");
        html!{
            <div id="login-container">
                <div id="username-label">{
                    "Username"
                }</div>
                <input id="username-input" oninput={self.set_username_callback()}/>
                <div id="password-label">{
                    "Password"
                }</div>
                <input id="password-input" type="password" oninput={self.set_password_callback()}/>
                <button id="show-password-button" onclick={self.toggle_show_password_callback()}>{
                    "Show"
                }</button>
                <button id="login-button">{
                    "Login"
                }</button>
                <button id="signup-button">{
                    "Signup"
                }</button>
                <button id="forgot-button">{
                    "Forgot login?"
                }</button>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateCredentials(update) => {
                console!(log, "UpdateCredentials");
                self.props.credentials =
                    self.props
                        .credentials
                        .clone()
                        .or(Some(Credentials::new()))
                        .map(move |mut c| {
                            update.update(&mut c);
                            c
                        });
                false
            },
            Msg::Login => {
                match self.props.credentials.clone() {
                    None => {},
                    Some(credentials) => {
                        // post login
                    },
                }
                true
            },
            Msg::ToggleShowPassword => {
                console!(log, "ToggleShowPassword");
                true
            }
        }
    }
}
