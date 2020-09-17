use warp::{
    ws::{
        WebSocket,
    },
};
use crate::{
    shared::{
        ServerMessage,
        ClientMessage,
    },
    Error,
};
use tracing::{
    debug,
    error,
};
use futures::{
    Stream,
    Sink,
    StreamExt,
    SinkExt,
    task::{
        Poll,
        Context,
    },
    stream::{
        SplitStream,
        SplitSink,
    },
};
use std::convert::{
    TryFrom,
    TryInto,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
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
use std::pin::Pin;
pub type SessionMap = HashMap<usize, Arc<RwLock<WebSocketSession>>>;
lazy_static! {
    static ref WEBSOCKETS: Arc<RwLock<SessionMap>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SOCKET_COUNT: AtomicUsize = AtomicUsize::new(0);
}
pub struct WebSocketSession {
    sender: SplitSink<WebSocket, warp::ws::Message>,
    receiver: SplitStream<WebSocket>,
}
impl WebSocketSession {
    pub fn new(sender: SplitSink<WebSocket, warp::ws::Message>, receiver: SplitStream<WebSocket>) -> Self {
        Self {
            sender,
            receiver,
        }
    }
    fn new_socket_id() -> usize {
        SOCKET_COUNT.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn send_update(&mut self) -> Result<(), Error> {
        self.send(ClientMessage::PriceHistory(Vec::new())).await
    }
    pub async fn receive_message(&mut self, msg: ServerMessage) -> Result<(), Error> {
        debug!("Received websocket msg");
        //debug!("{:#?}", msg);
        let response = match msg {
            ServerMessage::GetPriceHistory(req) => {
                Some(ClientMessage::PriceHistory(crate::binance().await.get_symbol_price_history(req).await?))
            },
            _ => None,
        };
        if let Some(response) = response {
            self.send(response).await.map_err(Into::into)
        } else {
            Ok(())
        }
    }
}
pub fn sessions() -> Arc<RwLock<SessionMap>> {
    WEBSOCKETS.clone()
}
pub async fn session(id: usize) -> Result<Arc<RwLock<WebSocketSession>>, Error> {
    WEBSOCKETS.clone()
        .read().await
        .get(&id)
        .ok_or(Error::WebSocket(format!("Websocket Connection ID {} not found!", id.clone())))
        .map(Clone::clone)
}
pub async fn handle_message(id: usize, msg: ServerMessage) -> Result<(), Error> {
    session(id).await?.write().await.receive_message(msg).await
}
pub async fn open_connection(websocket: WebSocket) -> Result<(), Error> {
    let id = WebSocketSession::new_socket_id();
    let (ws_sender, ws_receiver) = websocket.split();
    sessions().write().await
        .insert(id, Arc::new(RwLock::new(WebSocketSession::new(ws_sender, ws_receiver))));
    Ok(())
}
impl Stream for WebSocketSession {
    type Item = Result<ServerMessage, Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.receiver), cx)
            .map(|opt|
                opt.map(|res|
                    res.map_err(Into::into)
                    .and_then(|item| item.try_into())))
    }
}
impl Sink<ClientMessage> for WebSocketSession {
    type Error = Error;
    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn start_send(mut self: Pin<&mut Self>, item: ClientMessage) -> Result<(), Self::Error> {
        Sink::start_send(Pin::new(&mut self.sender), item.try_into()?).map_err(Into::into)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_close(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
}
impl TryFrom<warp::ws::Message> for ServerMessage {
    type Error = Error;
    fn try_from(msg: warp::ws::Message) -> Result<Self, Self::Error> {
        if let Ok(text) = msg.to_str() {
            serde_json::de::from_str(text).map_err(Into::into)
        } else {
            if msg.is_close() {
                Ok(ServerMessage::Close)
            } else if msg.is_ping() {
                Ok(ServerMessage::Ping)
            } else if msg.is_pong() {
                Ok(ServerMessage::Pong)
            } else if msg.is_binary() {
                let bytes = msg.as_bytes().to_vec();
                Ok(ServerMessage::Binary(bytes))
            } else {
                Err(Error::WebSocket(format!("Unable to read message: {:#?}", msg)))
            }
        }
    }
}
impl TryInto<warp::ws::Message> for ClientMessage {
    type Error = Error;
    fn try_into(self) -> Result<warp::ws::Message, Self::Error> {
        Ok(warp::ws::Message::text(
            serde_json::to_string(&self)
                .map_err(Error::SerdeJson)?)
        )
    }
}
