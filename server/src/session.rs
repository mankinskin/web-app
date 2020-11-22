
use std::collections::HashMap;
use async_std::{
    net::SocketAddr,
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};
use sha2::{
    Digest,
    Sha256,
};
use lazy_static::lazy_static;
lazy_static! {
    pub static ref SESSIONS: Arc<RwLock<SessionMap>> = Arc::new(RwLock::new(HashMap::new()));
}
async fn sessions() -> RwLockReadGuard<'static, SessionMap> {
    SESSIONS.read().await
}
async fn sessions_mut() -> RwLockWriteGuard<'static, SessionMap> {
    SESSIONS.write().await
}
const SECRET: &'static str = "change this";

pub type SessionID = String;
type SessionMap = HashMap<SocketAddr, SessionID>;
pub fn new_session_id(addr: &SocketAddr) -> SessionID {
    let mut h = Sha256::new();
    h.update(format!("{}{}", addr, SECRET));
    let out = h.finalize();
    hex::encode(out)
}
pub async fn get_session(addr: SocketAddr) -> SessionID {
    if let Some(id) = sessions().await.get(&addr) {
        return id.clone();
    }
    let id = new_session_id(&addr);
    sessions_mut().await.insert(addr, id.clone());
    id
}
