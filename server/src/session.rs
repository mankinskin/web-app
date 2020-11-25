
use std::collections::HashMap;
use async_std::{
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
use chrono::{
    DateTime,
    Duration,
    Utc,
};
// TODO: Replace with more general Account type
use app_model::User;
use std::ops::{
    Deref,
    DerefMut,
};
use rql::*;
use lazy_static::lazy_static;
use std::collections::VecDeque;
#[allow(unused)]
use tracing::{
    debug,
    warn,
    error,
};

lazy_static! {
    static ref SESSIONS: Arc<RwLock<SessionMap>> = Arc::new(RwLock::new(SessionMap::default()));
}
async fn sessions() -> RwLockReadGuard<'static, SessionMap> {
    SESSIONS.read().await
}
async fn sessions_mut() -> RwLockWriteGuard<'static, SessionMap> {
    SESSIONS.write().await
}
const SECRET: &'static str = "change this";
/// time until session expires
const EXPIRATION_SECS: u32 = 60;
/// time session remains valid after expiration
const STALE_SECS: u32 = 60;

pub type SessionID = String;
type InternalSessionMap = HashMap<SessionID, Session>;

#[derive(Debug, Default)]
pub struct SessionMap {
    sessions: InternalSessionMap,
    invalidations: VecDeque<(SessionID, DateTime<Utc>)>,
}
impl Deref for SessionMap {
    type Target = InternalSessionMap;
    fn deref(&self) -> &Self::Target {
        &self.sessions
    }
}
impl DerefMut for SessionMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sessions
    }
}
impl SessionMap {
    fn new_id() -> SessionID {
        let mut h = Sha256::new();
        let timestamp = Utc::now().timestamp_nanos();
        h.update(format!("{}{}", timestamp, SECRET));
        let out = h.finalize();
        hex::encode(out)
    }
    fn get_session(&self, id: &SessionID) -> Option<&Session> {
        self.sessions.get(id)
    }
    fn remove_session(&mut self, id: &SessionID) {
        debug!("Removing session {}", id);
        self.sessions.remove(id);
    }
    fn create_session(&mut self) -> Session {
        let id = Self::new_id();
        let session = Session::new(id.clone());
        self.sessions.insert(id.clone(), session.clone());
        self.invalidations.push_back((id, session.invalidation_time().clone()));
        session
    }
}
pub async fn create_session() -> Session {
    sessions_mut().await.create_session()
}
pub async fn get_session(id: &SessionID) -> Option<Session> {
    sessions().await.get_session(id).map(Clone::clone)
}
pub async fn run_cleaner() {
    loop {
        let invalidation = sessions_mut().await.invalidations.pop_front().clone();
        if let Some((next, time)) = invalidation {
            let duration = time - Utc::now();
            let duration = tokio::time::Duration::from_millis(duration.num_milliseconds() as u64);
            tokio::time::delay_for(duration).await;
            sessions_mut().await.remove_session(&next);
        }
    }
}
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionID,
    user: Option<Id<User>>,
    created: DateTime<Utc>,
}

impl Session {
    pub fn expiration_time(&self) -> DateTime<Utc> {
        self.created + Duration::seconds(EXPIRATION_SECS.into())
    }
    pub fn invalidation_time(&self) -> DateTime<Utc> {
        self.expiration_time() + Duration::seconds(STALE_SECS.into())
    }
    pub fn new(id: SessionID) -> Self {
        Self {
            id,
            user: None,
            created: Utc::now(),
        }
    }
    pub fn with_user(id: SessionID, user: Id<User>) -> Self {
        Self {
            user: Some(user),
            ..Self::new(id)
        }
    }
    /// session is stale in [expiration, invalidation)
    pub fn is_stale(&self) -> bool {
        self.is_valid() &&
        self.expiration_time() <= Utc::now()
    }
    /// session is valid in [creation, invalidation)
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.invalidation_time()
    }
    pub fn cookie_string(&self) -> String {
        format!("session={}; Max-Age={}; Secure",
                self.id,
                (self.invalidation_time() - Utc::now()).num_seconds(),
        )
    }
}
