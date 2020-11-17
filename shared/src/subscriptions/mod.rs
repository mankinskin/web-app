use openlimits::model::{
    Interval,
};
use serde::{
    Deserialize,
    Serialize,
};
use database_table::Entry;
use app_model::market::PriceHistory;

#[cfg(all(feature = "actix_server", not(target_arch = "wasm32")))]
use {
    actix::Message,
};
use rql::*;
use enum_paths::AsPath;

pub mod subscription;
pub use subscription::PriceSubscription;

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    #[as_path = ""]
    Entry(Id<PriceSubscription>),
    #[as_path = ""]
    List,
}
#[cfg(target_arch = "wasm32")]
impl database_table::Route for Route {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePriceSubscriptionRequest {
    pub interval: Option<Interval>,
}
impl From<Interval> for UpdatePriceSubscriptionRequest {
    fn from(interval: Interval) -> Self {
        Self {
            interval: Some(interval),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), derive(Message))]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), rtype(result = "Option<Response>"))]
pub enum Request {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceSubscription),
    Subscription(Id<PriceSubscription>, SubscriptionRequest),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), derive(Message))]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), rtype(result = "Option<Response>"))]
pub enum SubscriptionRequest {
    UpdatePriceSubscription(UpdatePriceSubscriptionRequest),
    StartHistoryUpdates,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), derive(Message))]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), rtype(result = "()"))]
pub enum Response {
    SubscriptionList(Vec<Entry<PriceSubscription>>),
    PriceHistory(Id<PriceSubscription>, PriceHistory),
    SubscriptionAdded(Id<PriceSubscription>),
    SubscriptionNotFound(Id<PriceSubscription>),
    SubscriptionUpdated,
}
