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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    PriceHistory(PriceHistory),
}
