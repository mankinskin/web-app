pub mod subscriptions;
pub use subscriptions::PriceSubscription;

use serde::{
    Deserialize,
    Serialize,
};
use enum_paths::AsPath;
use app_model::{
    user::Route as UserRoute,
    auth::Route as AuthRoute,
};
#[cfg(all(feature = "actix_server", not(target_arch = "wasm32")))]
use {
    actix::Message,
};
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), derive(Message))]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), rtype(result = "Option<ServerMessage>"))]
pub enum ClientMessage {
    Subscriptions(subscriptions::Request),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}
#[cfg(all(feature = "warp_server", not(target_arch = "wasm32")))]
use {
    std::convert::TryFrom,
};
#[cfg(all(feature = "warp_server", not(target_arch = "wasm32")))]
impl TryFrom<warp::ws::Message> for ClientMessage {
    type Error = String;
    fn try_from(msg: warp::ws::Message) -> Result<Self, Self::Error> {
        if let Ok(text) = msg.to_str() {
            serde_json::de::from_str(text).map_err(|e| e.to_string())
        } else {
            if msg.is_close() {
                Ok(Self::Close)
            } else if msg.is_ping() {
                Ok(Self::Ping)
            } else if msg.is_pong() {
                Ok(Self::Pong)
            } else if msg.is_binary() {
                let bytes = msg.as_bytes().to_vec();
                Ok(Self::Binary(bytes))
            } else {
                Err(format!("Unable to read message: {:#?}", msg))
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), derive(Message))]
#[cfg_attr(all(feature = "actix_server", not(target_arch = "wasm32")), rtype(result = "()"))]
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
impl database_table::Route for Route {}

#[cfg(target_arch = "wasm32")]
impl components::Route for Route {}

#[derive(Clone, Debug, AsPath)]
pub enum ApiRoute {
    Subscriptions(subscriptions::Route),
}
impl database_table::Route for ApiRoute {}

impl database_table::Routable for ApiRoute {
    type Route = Self;
    fn route(&self) -> Self::Route {
        self.clone()
    }
}
