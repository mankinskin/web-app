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
use rql::{
    *,
};
use url::{
    *,
};
use common::{
    fetch::*,
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
pub struct SignupData {
    pub signup: Url,
    pub user: Option<User>,
}
pub struct Signup {
    link: ComponentLink<Self>,
    props: SignupData,
}
pub enum Msg {
    SignupResponse(FetchResponse<Id<User>>),
    UpdateUser(UserUpdate),
    ToggleShowPassword,
    Signup,
}

impl Signup {
    fn set_username_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::UpdateUser(
                User::update()
                    .name(input.value)
            )
        })
    }
    fn set_password_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::UpdateUser(
                User::update()
                .password(input.value)
            )
        })
    }
    fn toggle_show_password_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(|_: ClickEvent| {
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
            ).unwrap();
            Msg::ToggleShowPassword
        })
    }
    fn signup_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(|_: ClickEvent| {
            Msg::Signup
        })
    }
    fn signup_responder(&self) -> Callback<FetchResponse<Id<User>>> {
        self.link.callback(|response| {
            Msg::SignupResponse(response)
        })
    }
}
impl Component for Signup {
    type Message = Msg;
    type Properties = SignupData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let s = Self {
            props,
            link,
        };
        s
    }
    fn view(&self) -> Html {
        let user = self.props.user.clone().unwrap_or(User::empty());
        let credentials = user.credentials();
        html!{
            <div id="login-container">
                <div id="username-label">{
                    "Username"
                }</div>
                <input id="username-input" oninput={self.set_username_callback()}/>
                <div id="username-invalid-icon">{
                    format!("{}", credentials.username_is_valid())
                }</div>
                <div id="username-invalid-text">{
                    credentials.username_invalid_text()
                }</div>
                <div id="password-label">{
                    "Password"
                }</div>
                <input id="password-input" type="password" oninput={self.set_password_callback()}/>
                <button id="show-password-button" onclick={self.toggle_show_password_callback()}>{
                    "Show"
                }</button>
                <div id="password-invalid-icon">{
                    format!("{}", credentials.password_is_valid())
                }</div>
                <div id="password-invalid-text">{
                    credentials.password_invalid_text()
                }</div>
                <button id="signup-button" onclick={self.signup_callback()}>{
                    "Signup"
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
            Msg::UpdateUser(update) => {
                console!(log, "UpdateCredentials");
                self.props.user =
                    self.props
                        .user
                        .clone()
                        .or(Some(User::empty()))
                        .map(move |mut c| {
                            update.update(&mut c);
                            c
                        });
                true
            },
            Msg::ToggleShowPassword => {
                console!(log, "ToggleShowPassword");
                true
            },
            Msg::Signup => {
                match self.props.user.clone() {
                    None => {
                        // Message "Fill in credentials"
                    },
                    Some(user) => {
                        // post login
                        Fetch::post(self.props.signup.clone(), user)
                            .responder(self.signup_responder())
                            .send()
                            .expect("Signup Request failed");
                    },
                }
                true
            },
            Msg::SignupResponse(response) => {
                console!(log, format!("Response: {:?}", response));
                match response.into_inner() {
                    Ok(_id) => {
                    },
                    Err(e) => {
                        console!(log, format!("Error: {}", e));
                    },
                }
                true
            },
        }
    }
}
