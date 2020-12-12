use crate::auth::UserSession;
use components::{
    Component,
    Viewable,
};
use lazy_static::lazy_static;
use seed::{
    browser::web_storage::{
        SessionStorage,
        WebStorage,
    },
    prelude::*,
    *,
};
use std::sync::Mutex;
const STORAGE_KEY: &str = "session";
lazy_static! {
    static ref SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
}
pub fn load() -> Option<UserSession> {
    SessionStorage::get(STORAGE_KEY).ok()
}
pub fn store(session: &UserSession) {
    SessionStorage::insert(STORAGE_KEY, session).expect("insert into session storage failed")
}
pub fn clear() {
    SessionStorage::clear().expect("clearing session storage failed")
}
pub fn set(session: UserSession) {
    seed::log!("Setting UserSession");
    *SESSION.lock().unwrap() = Some(session.clone());
    store(&session.clone());
}
pub fn get() -> Option<UserSession> {
    SESSION.lock().unwrap().clone()
}
pub fn end() {
    *SESSION.lock().unwrap() = None;
    clear();
}
#[derive(Debug, Clone, Default)]
pub struct Session;
impl Session {
    async fn logout_request(self) -> Result<(), FetchError> {
        let req = seed::fetch::Request::new(format!("{}/api/auth/logout", crate::get_base_url().unwrap())).method(Method::Post);
        seed::fetch::fetch(req)
            .await?
            .check_status()?;
        Ok(())
    }
}
impl From<UserSession> for Session {
    fn from(session: UserSession) -> Self {
        set(session);
        Self
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Logout,
}
impl Component for Session {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Logout => {
                orders.perform_cmd(self.clone().logout_request());
            },
        }
    }
}
impl Viewable for Session {
    fn view(&self) -> Node<Msg> {
        if let Some(session) = get() {
            div![
                p![format!("Logged in as {:#?}", session.user_id)],
                button![
                    ev(Ev::Click, |_event| Msg::Logout),
                    ["Logout"],

                ]
            ]
        } else {
            empty![]
        }
    }
}
