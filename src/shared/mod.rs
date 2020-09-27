use serde::{
    Serialize,
    Deserialize,
};
use openlimits::{
    model::{
        Candle,
        Interval,
    },
};

#[cfg(not(target_arch = "wasm32"))]
use {
    std::{
        convert::{
            TryFrom,
            TryInto,
        },
    },
    crate::Error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<openlimits::model::Interval>,
    pub paginator: Option<openlimits::model::Paginator<u64>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistory {
    pub market_pair: String,
    pub candles: Vec<Candle>,
    pub time_interval: Interval,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    SubscribePrice(String),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}
#[cfg(not(target_arch = "wasm32"))]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    PriceHistory(PriceHistory),
}
#[cfg(not(target_arch = "wasm32"))]
impl TryInto<warp::ws::Message> for ClientMessage {
    type Error = Error;
    fn try_into(self) -> Result<warp::ws::Message, Self::Error> {
        Ok(warp::ws::Message::text(
            serde_json::to_string(&self)
                .map_err(Error::SerdeJson)?)
        )
    }
}
