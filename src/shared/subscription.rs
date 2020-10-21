use crate::shared::PriceHistoryRequest;
use openlimits::model::{
    Paginator,
    Interval,
};
use serde::{
    Deserialize,
    Serialize,
};

use app_model::market::PriceHistory;

#[cfg(not(target_arch = "wasm32"))]
use {
    crate::binance::Binance,
    std::result::Result,
    actix::Message,
};
use rql::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceSubscription {
    pub market_pair: String,
    pub time_interval: Interval,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}
impl From<PriceHistoryRequest> for PriceSubscription {
    fn from(request: PriceHistoryRequest) -> Self {
        Self {
            market_pair: request.market_pair,
            time_interval: request.interval.unwrap_or(Interval::OneMinute),
            last_update: None,
        }
    }
}

impl PartialEq<&PriceHistoryRequest> for PriceSubscription {
    fn eq(&self, rhs: &&PriceHistoryRequest) -> bool {
        self.market_pair == *rhs.market_pair
    }
}
impl PriceSubscription {
    pub fn paginator(&self) -> Option<Paginator<u32>> {
        self.last_update.map(|datetime| {
            Paginator {
                start_time: Some(datetime.timestamp_millis() as u64),
                ..Default::default()
            }
        })
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl PriceSubscription {
    pub async fn is_available(&self) -> bool {
        let symbol = self.market_pair.to_uppercase();
        crate::binance::Binance::symbol_available(&symbol).await
    }
    pub async fn get_price_history_request(&self) -> PriceHistoryRequest {
        let paginator = self.paginator();
        PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.time_interval),
            paginator,
        }
    }
    pub async fn latest_price_history(&self) -> Result<PriceHistory, crate::binance::Error> {
        let req = self.get_price_history_request().await;
        Binance::get_symbol_price_history(req).await.map_err(|e| e.to_string().into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "Option<Response>"))]
pub enum Request {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceHistoryRequest),
    GetHistoryUpdates(Id<PriceSubscription>),
    SendPriceHistory(Id<PriceSubscription>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "()"))]
pub enum Response {
    SubscriptionList(HashMap<Id<PriceSubscription>, PriceSubscription>),
    PriceHistory(Id<PriceSubscription>, PriceHistory),
    SubscriptionAdded(Id<PriceSubscription>),
}
