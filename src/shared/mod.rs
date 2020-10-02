use serde::{
    Serialize,
    Deserialize,
};

use openlimits::{
    model::{
        Paginator,
        Interval,
    },
};
use app_model::{
    market::{
        PriceHistory,
    },
};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
    pub paginator: Option<Paginator<u64>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    SubscribePrice(String),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    PriceHistory(PriceHistory),
}
#[cfg(not(target_arch = "wasm32"))]
use std::{
    convert::{
        TryFrom,
        TryInto,
    },
};

#[cfg(not(target_arch = "wasm32"))]
impl TryInto<warp::ws::Message> for ClientMessage {
    type Error = crate::Error;
    fn try_into(self) -> Result<warp::ws::Message, Self::Error> {
        Ok(warp::ws::Message::text(
            serde_json::to_string(&self)
                .map_err(crate::Error::SerdeJson)?)
        )
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<warp::ws::Message> for ServerMessage {
    type Error = crate::Error;
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
                Err(crate::Error::WebSocket(format!("Unable to read message: {:#?}", msg)))
            }
        }
    }
}
