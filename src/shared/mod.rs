pub mod subscription;
pub use subscription::PriceSubscription;

use serde::{
    Deserialize,
    Serialize,
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
