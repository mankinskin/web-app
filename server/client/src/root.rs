use yew::{
    *,
};
use crate::{
    router::*,
};
use plans::{
    user::*,
};

pub enum Msg {
    SetSession(UserSession),
}

pub struct ClientRoot {
    link: ComponentLink<Self>,
    session: Option<UserSession>,
}
impl ClientRoot {
    fn session_setter(&self) -> Callback<UserSession> {
        self.link.callback(move |session| {
            Msg::SetSession(session)
        })
    }
    pub fn set_session(&mut self, session: UserSession) {
        self.session = Some(session);
    }
}

impl Component for ClientRoot {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            session: None,
        }
    }
    fn view(&self) -> Html {
        html!{
            <ClientRouter
                session_setter={self.session_setter()}
                session={self.session.clone()}
            />
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetSession(session) => {
                console!(log, format!("Session set: {:#?}", session));
                self.set_session(session);
            },
        }
        true
    }
}
