use app_model::market::PriceHistory;
use database_table::Routable;
use database_table::{
    Entry,
    Route as DbRoute,
};
use openlimits::model::Interval;
use serde::{
    Deserialize,
    Serialize,
};

use enum_paths::AsPath;
use rql::*;

pub mod subscription;
pub use subscription::PriceSubscription;

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    #[as_path = ""]
    Entry(Id<PriceSubscription>),
    #[as_path = ""]
    List,
}
impl DbRoute for Route {}

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
pub enum Request {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceSubscription),
    Subscription(Id<PriceSubscription>, SubscriptionRequest),
}

impl Routable for Request {
    type Route = Route;
    fn route(&self) -> Self::Route {
        match self {
            Request::GetPriceSubscriptionList => Route::List,
            Request::AddPriceSubscription(_) => Route::List,
            Request::Subscription(id, _) => Route::Entry(id.clone()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubscriptionRequest {
    UpdatePriceSubscription(UpdatePriceSubscriptionRequest),
    StartHistoryUpdates,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    SubscriptionList(Vec<Entry<PriceSubscription>>),
    PriceHistory(Id<PriceSubscription>, PriceHistory),
    SubscriptionAdded(Id<PriceSubscription>),
    SubscriptionNotFound(Id<PriceSubscription>),
    SubscriptionUpdated,
}
