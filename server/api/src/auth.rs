
use std::sync::{
    Mutex,
};
use seed::browser::web_storage::{
    WebStorage,
    SessionStorage,
};
use plans::{
    user::*,
};
lazy_static! {
    static ref USER_SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
}
const STORAGE_KEY: &str = "secret";
pub fn load_session() -> Option<UserSession> {
    SessionStorage::get(STORAGE_KEY).ok()
}
pub fn store_session(session: &UserSession) {
    SessionStorage::insert(STORAGE_KEY, session)
        .expect("insert into session storage failed")
}
pub fn clear_session() {
    SessionStorage::clear()
        .expect("clearing session storage failed")
}
pub fn set_session(session: UserSession) {
    *USER_SESSION.lock().unwrap() = Some(session.clone());
    store_session(&session.clone());
}
pub fn get_session() -> Option<UserSession> {
    USER_SESSION.lock().unwrap().clone()
}
pub fn end_session() {
    *USER_SESSION.lock().unwrap() = None;
    clear_session();
}
