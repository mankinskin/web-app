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
use tokio::{
    stream::StreamMap,
};


pub type ConnectionMap = StreamMap<usize, ConnectionStream>;
lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<ConnectionMap>> = Arc::new(RwLock::new(ConnectionMap::new()));
    static ref CONNECTION_IDS: AtomicUsize = AtomicUsize::new(0);
}
pub struct ConnectionStream(Arc<RwLock<Connection>>);
impl Stream for ConnectionStream {
    type Item = Result<ServerMessage, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        if let Some(mut connection) = (*self.0).try_write() {
            Stream::poll_next(Pin::new(&mut *connection), cx)
        } else {
            Poll::Pending
        }
    }
}
pub struct Connections;
impl Connections {
    fn new_id() -> usize {
        CONNECTION_IDS.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn add(connection: Connection) -> usize {
        let id = Self::new_id();
        debug!("Opening Websocket connection {}", id);
        (*CONNECTIONS).write().await.insert(id, ConnectionStream(Arc::new(RwLock::new(connection))));
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
        if let Some(mut connections) = CONNECTIONS.try_write() {
            Stream::poll_next(Pin::new(&mut *connections), cx)
                .map(|opt| opt.map(|(id, res)| res.map(|msg| Message::WebSocket(id, msg))))
        } else {
            Poll::Pending
        }
    }
}
