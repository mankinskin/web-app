#[cfg(not(feature = "actix_server"))]
pub mod riker_actor;
#[cfg(not(feature = "actix_server"))]
pub use riker_actor as actor;

#[cfg(feature = "actix_server")]
pub mod actix_actor;
#[cfg(feature = "actix_server")]
pub use actix_actor as actor;

pub use actor::Session;
#[cfg(not(feature = "actix_server"))]
pub use actor::websocket_session;

#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};
use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};
use lazy_static::lazy_static;
lazy_static! {
    static ref SESSION_COUNT: AtomicUsize = AtomicUsize::new(0);
}

pub fn new_session_id() -> usize {
    SESSION_COUNT.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct Error(String);
impl<E: ToString> From<E> for Error {
    fn from(s: E) -> Self {
        Self(s.to_string())
    }
}
