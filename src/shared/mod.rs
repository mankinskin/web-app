pub mod subscription;
pub use subscription::PriceSubscription;

use serde::{
    Deserialize,
    Serialize,
};

use app_model::market::PriceHistory;
use openlimits::model::{
    Interval,
    Paginator,
};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::server::websocket::Error,
    actix::Message,
    std::convert::{
        TryFrom,
        TryInto,
    }
};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
    pub paginator: Option<Paginator<u32>>,
}
impl From<String> for PriceHistoryRequest {
    fn from(market_pair: String) -> Self {
        Self {
            market_pair,
            interval: None,
            paginator: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "Option<ServerMessage>"))]
pub enum ClientMessage {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceHistoryRequest),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "()"))]
pub enum ServerMessage {
    PriceHistory(PriceHistory),
    SubscriptionList(HashMap<usize, PriceSubscription>),
    SubscriptionAdded(usize),
}
