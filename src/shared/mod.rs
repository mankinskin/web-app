pub mod subscription;
pub use subscription::PriceSubscription;

use serde::{
    Deserialize,
    Serialize,
};
use openlimits::model::{
    Interval,
    Paginator,
};
use enum_paths::AsPath;
use app_model::{
    user::Route as UserRoute,
    auth::Route as AuthRoute,
};
#[cfg(not(target_arch = "wasm32"))]
use {
    actix::Message,
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
    Subscriptions(subscription::Request),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Message))]
#[cfg_attr(not(target_arch = "wasm32"), rtype(result = "()"))]
pub enum ServerMessage {
    Subscriptions(subscription::Response),
}

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Subscriptions,
    #[as_path = ""]
    Auth(AuthRoute),
    #[as_path = ""]
    User(UserRoute),
    #[as_path = ""]
    Root,
}
impl database_table::Route for Route {}

impl database_table::Routable for Route {
    type Route = Self;
    fn route(&self) -> Self::Route {
        self.clone()
    }
}
