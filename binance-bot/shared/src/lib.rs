pub mod subscriptions;
pub use subscriptions::PriceSubscription;

use app_model::{
    auth::Route as AuthRoute,
    user::Route as UserRoute,
};
use enum_paths::AsPath;
use serde::{
    Deserialize,
    Serialize,
};
use database_table::{
    Route as DbRoute,
    Routable,
};
use std::convert::TryFrom;
#[derive(Debug, Clone)]
pub enum WebsocketCommand {
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
    ClientMessage(ClientMessage),
    ServerMessage(ServerMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Subscriptions(subscriptions::Request),
}
impl Routable for ClientMessage {
    type Route = Route;
    fn route(&self) -> Self::Route {
        match self {
            ClientMessage::Subscriptions(req) => Route::Subscriptions(req.route()),
        }
    }
}

impl TryFrom<String> for ClientMessage {
    type Error = String;
    fn try_from(msg: String) -> Result<Self, Self::Error> {
        serde_json::de::from_str(&msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Subscriptions(subscriptions::Response),
}

#[derive(Clone, Debug, AsPath)]
pub enum Route {
    Api(ApiRoute),
    Subscriptions(subscriptions::Route),
    #[as_path = ""]
    Auth(AuthRoute),
    #[as_path = ""]
    User(UserRoute),
    #[as_path = ""]
    Root,
}
impl Default for Route {
    fn default() -> Self {
        Self::Root
    }
}
impl DbRoute for Route {}

#[cfg(target_arch = "wasm32")]
impl components::Route for Route {}

#[derive(Clone, Debug, AsPath)]
pub enum ApiRoute {
    Subscriptions(subscriptions::Route),
}
impl DbRoute for ApiRoute {}

impl Routable for ApiRoute {
    type Route = Self;
    fn route(&self) -> Self::Route {
        self.clone()
    }
}
