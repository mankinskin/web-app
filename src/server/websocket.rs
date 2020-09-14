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
};
use futures::{
    StreamExt,
    SinkExt,
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
lazy_static! {
    pub static ref WEBSOCKETS: Arc<RwLock<HashMap<usize, (SplitSink<WebSocket, warp::ws::Message>, SplitStream<WebSocket>)>>> = Arc::new(RwLock::new(HashMap::new()));
    pub static ref SOCKET_COUNT: AtomicUsize = AtomicUsize::new(0);
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
fn new_socket_id() -> usize {
    SOCKET_COUNT.fetch_add(1, Ordering::Relaxed)
}
pub async fn open(websocket: WebSocket) -> Result<(), Error> {
    debug!("Opening WebSocket");

    WEBSOCKETS.try_write()
        .expect("Failed to write to WEBSOCKETS")
        .insert(new_socket_id(), websocket.split());

    Ok(())
}
pub async fn close(id: usize) -> Result<(), Error> {
    debug!("Closing WebSocket connection");
    WEBSOCKETS.try_write().unwrap().remove(&id);
    Ok(())
}
pub async fn handle_message(id: usize, msg: warp::ws::Message) -> Result<(), Error> {
    debug!("Received websocket msg");
    //debug!("{:#?}", msg);
    let msg = ServerMessage::try_from(msg)?;
    let response = match msg {
        ServerMessage::GetPriceHistory(req) => {
            Some(ClientMessage::PriceHistory(crate::binance().await.get_symbol_price_history(req).await?))
        },
        ServerMessage::Close => {
            close(id).await?;
            None
        },
        _ => None,
    };
    if let Some(response) = response {
        let mut handle = WEBSOCKETS.try_write().unwrap();
        let (tx, _) = handle.get_mut(&id).unwrap();
        tx.send(response.try_into()?).await?;
    }
    Ok(())
}
