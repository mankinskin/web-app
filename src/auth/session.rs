use components::{
    Component,
    Viewable,
};
use crate::auth::UserSession;
use lazy_static::lazy_static;
use seed::{
    browser::web_storage::{SessionStorage, WebStorage},
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

impl From<UserSession> for Session {
    fn from(session: UserSession) -> Self {
        set(session);
        Self
    }
}
#[derive(Clone, Debug)]
pub enum Msg {}
impl Component for Session {
    type Msg = Msg;
    fn update(&mut self, _msg: Self::Msg, _orders: &mut impl Orders<Self::Msg>) {}
}
impl Viewable for Session {
    fn view(&self) -> Node<Msg> {
        if let Some(session) = get() {
            div![p!["logged in"], p![format!("{:#?}", session.user_id)],]
        } else {
            empty![]
        }
    }
}
