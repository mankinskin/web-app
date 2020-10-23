use openlimits::model::{
    Interval,
};
use serde::{
    Deserialize,
    Serialize,
};
use database_table::Entry;
use app_model::market::PriceHistory;

#[cfg(not(target_arch = "wasm32"))]
use {
    actix::Message,
};
use rql::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceSubscriptionRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
}
impl From<String> for PriceSubscriptionRequest {
    fn from(market_pair: String) -> Self {
        Self {
            market_pair,
            interval: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "Option<Response>"))]
pub enum Request {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceSubscriptionRequest),
    UpdatePriceSubscription(PriceSubscriptionRequest),
    StartHistoryUpdates(Id<PriceSubscription>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "()"))]
pub enum Response {
    SubscriptionList(Vec<Entry<PriceSubscription>>),
    PriceHistory(Id<PriceSubscription>, PriceHistory),
    SubscriptionAdded(Id<PriceSubscription>),
    SubscriptionUpdated,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceSubscription {
    pub market_pair: String,
    pub interval: Option<Interval>,
}
impl From<PriceSubscriptionRequest> for PriceSubscription {
    fn from(request: PriceSubscriptionRequest) -> Self {
        Self {
            market_pair: request.market_pair,
            interval: request.interval,
        }
    }
}
impl PartialEq<&PriceSubscriptionRequest> for PriceSubscription {
    fn eq(&self, rhs: &&PriceSubscriptionRequest) -> bool {
        self.market_pair == *rhs.market_pair
    }
}
