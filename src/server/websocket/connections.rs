use super::connection::Connection;
use crate::{
    shared::{
        ClientMessage,
        ServerMessage,
    },
    websocket::Error,
};
use async_std::sync::{
    Arc,
    RwLock,
};
use futures::{
    task::{
        Context,
        Poll,
    },
    Sink,
    SinkExt,
    Stream,
};
use lazy_static::lazy_static;
use std::pin::Pin;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use tokio::stream::StreamMap;
#[allow(unused)]
use tracing::{
    debug,
    error,
};

pub type ConnectionMap = StreamMap<usize, ConnectionStream>;
lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<ConnectionMap>> =
        Arc::new(RwLock::new(ConnectionMap::new()));
    static ref CONNECTION_IDS: AtomicUsize = AtomicUsize::new(0);
}
#[derive(Clone, Debug)]
pub struct ConnectionStream(Arc<RwLock<Connection>>);
impl Stream for ConnectionStream {
    type Item = Result<ClientMessage, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        if let Some(mut connection) = (*self.0).try_write() {
            Stream::poll_next(Pin::new(&mut *connection), cx)
        } else {
            Poll::Pending
        }
    }
}
impl Sink<ServerMessage> for ConnectionStream {
    type Error = Error;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if let Some(mut connection) = (*self.0).try_write() {
            Sink::poll_ready(Pin::new(&mut *connection), cx).map_err(Into::into)
        } else {
            Poll::Pending
        }
    }
    fn start_send(self: Pin<&mut Self>, item: ServerMessage) -> Result<(), Self::Error> {
        Sink::start_send(Pin::new(&mut *self.try_write().unwrap()), item).map_err(Into::into)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if let Some(mut connection) = (*self.0).try_write() {
            Sink::poll_flush(Pin::new(&mut *connection), cx).map_err(Into::into)
        } else {
            Poll::Pending
        }
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if let Some(mut connection) = (*self.0).try_write() {
            Sink::poll_close(Pin::new(&mut *connection), cx).map_err(Into::into)
        } else {
            Poll::Pending
        }
    }
}
use std::ops::{
    Deref,
    DerefMut,
};
impl Deref for ConnectionStream {
    type Target = Arc<RwLock<Connection>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ConnectionStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
        (*CONNECTIONS)
            .write()
            .await
            .insert(id, ConnectionStream(Arc::new(RwLock::new(connection))));
        id
    }
    pub async fn remove(id: usize) {
        (*CONNECTIONS).write().await.remove(&id);
        debug!("Closed WebSocket connection {}", id);
    }
    pub async fn contains(id: usize) -> bool {
        let connections = (*CONNECTIONS).read().await;
        connections.contains_key(&id)
    }
    pub async fn connection(id: usize) -> Option<ConnectionStream> {
        let mut connections = (*CONNECTIONS).write().await;
        // TODO replace with iter_mut once pull request got accepted and published
        let index = if let Some(index) = connections
            .keys()
            .enumerate()
            .find_map(|(index, k)| (*k == id).then_some(index.clone()))
        {
            index
        } else {
            return None;
        };
        let result = connections
            .values_mut()
            .enumerate()
            .find_map(|(i, v)| (i == index).then_some(v.clone()));
        result
    }
    pub async fn send_all(msg: ServerMessage) {
        for connection in CONNECTIONS.write().await.values_mut() {
            connection
                .write()
                .await
                .send(msg.clone())
                .await
                .expect("Error when sending messages")
        }
    }
}
#[derive(Debug, Clone)]
pub struct ConnectionClientMessage(pub usize, pub ClientMessage);

impl Stream for Connections {
    type Item = Result<ConnectionClientMessage, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        if let Some(mut connections) = CONNECTIONS.try_write() {
            //debug!("Polling Connections");
            let poll = Stream::poll_next(Pin::new(&mut *connections), cx)
                .map(|opt| opt.map(|(id, res)| res.map(|msg| ConnectionClientMessage(id, msg))));
            if let Poll::Ready(None) = poll {
                Poll::Pending
            } else {
                poll
            }
        } else {
            Poll::Pending
        }
    }
}
