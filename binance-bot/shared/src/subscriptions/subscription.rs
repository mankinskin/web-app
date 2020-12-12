use serde::{
    Deserialize,
    Serialize,
};

#[cfg(target_arch = "wasm32")]
use {
    crate::{
        subscriptions::Route,
        ApiRoute,
        Route as CrateRoute,
    },
    components::{
        Component,
        Edit,
    },
    database_table::TableRoutable,
    rql::*,
    seed::{
        prelude::*,
        *,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PriceSubscription {
    pub market_pair: String,
}
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub enum PriceSubscriptionMsg {
    SetMarketPair(String),
}
#[cfg(target_arch = "wasm32")]
impl Component for PriceSubscription {
    type Msg = PriceSubscriptionMsg;
    fn update(&mut self, msg: Self::Msg, _orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Self::Msg::SetMarketPair(s) => self.market_pair = s,
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Edit for PriceSubscription {
    fn edit(&self) -> Node<Self::Msg> {
        div![
            label!["market_pair"],
            input![
                attrs! {
                    At::Placeholder => "market_pair",
                    At::Value => self.market_pair,
                },
                input_ev(Ev::Input, Self::Msg::SetMarketPair)
            ],
        ]
    }
}
#[cfg(target_arch = "wasm32")]
impl TableRoutable for PriceSubscription {
    type Route = CrateRoute;
    fn table_route() -> Self::Route {
        CrateRoute::Api(ApiRoute::Subscriptions(Route::List))
    }
    fn entry_route(id: Id<Self>) -> Self::Route {
        CrateRoute::Api(ApiRoute::Subscriptions(Route::Entry(id)))
    }
}

impl From<String> for PriceSubscription {
    fn from(market_pair: String) -> Self {
        Self { market_pair }
    }
}
