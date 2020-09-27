use crate::{
    shared::{
        ServerMessage,
    },
    Error,
    message_stream::Message,
};
use futures::{
    Stream,
    task::{
        Poll,
    },
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
};
use async_std::{
    sync::{
        Arc,
        RwLock,
    },
};
#[allow(unused)]
use tracing::{
    debug,
    error,
};
use std::pin::Pin;
use super::{
    connection::Connection,
};


pub type ConnectionMap = HashMap<usize, Arc<RwLock<Connection>>>;
lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<ConnectionMap>> = Arc::new(RwLock::new(ConnectionMap::new()));
    static ref CONNECTION_IDS: AtomicUsize = AtomicUsize::new(0);
}
pub struct Connections;
impl Connections {
    fn new_id() -> usize {
        CONNECTION_IDS.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn add(connection: Connection) -> usize {
        let id = Self::new_id();
        debug!("Opening Websocket connection {}", id);
        (*CONNECTIONS).write().await.insert(id, Arc::new(RwLock::new(connection)));
        id
    }
    pub async fn remove(id: usize) {
        (*CONNECTIONS).write().await.remove(&id);
        debug!("Closed WebSocket connection {}", id);
    }
    //pub async fn push_updates() {
    //    for (_, connection) in CONNECTIONS
    //        .write()
    //        .await
    //        .iter_mut() {
    //        connection.write()
    //            .await
    //            .push_update()
    //            .await
    //            .expect("Error when pushing updates")
    //    }
    //}
}
impl Stream for Connections {
    type Item = Result<Message, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        //debug!("Polling Connections");
        CONNECTIONS.try_write()
            .and_then(|mut connections| {
                //debug!("writing on {} connections", connections.len());
                connections.iter_mut()
                    .find_map(|(id, session)| { // find connection ready
                        //debug!("checking session {}", id);
                        session.try_write().and_then(|mut session| {
                            //debug!("polling session {}", id);
                            let session_poll = Stream::poll_next(Pin::new(&mut *session), cx);
                            if let Poll::Ready(opt) = session_poll {
                                //debug!("session {} ready", id);
                                opt.map(|result| {
                                    result
                                        .map(|msg| {
                                            if let ServerMessage::Close = msg {
                                            }
                                            Message::WebSocket(id.clone(), msg)
                                        })
                                        .map_err(Error::from)
                                })
                            } else {
                                None
                            }
                        })
                    })
            })
            .map(|res| Poll::Ready(Some(res)))
            .unwrap_or(Poll::Pending)
    }
}
