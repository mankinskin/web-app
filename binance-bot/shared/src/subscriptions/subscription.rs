use serde::{
    Deserialize,
    Serialize,
};

use {
    crate::{
        subscriptions::Route,
        ApiRoute,
        Route as CrateRoute,
    },
    database_table::{
        Routed,
        TableRoutable,
    },
    rql::*,
};
#[cfg(target_arch = "wasm32")]
use {
    components::{
        Component,
        Edit,
    },
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
impl TableRoutable for PriceSubscription {
    type Route = Route;
    fn table_route() -> Self::Route {
        Route::List
    }
    fn entry_route(id: Id<Self>) -> Self::Route {
        Route::Entry(id)
    }
}
impl Routed for PriceSubscription {
    type AbsoluteRoute = CrateRoute;
    fn to_absolute_route(route: <Self as TableRoutable>::Route) -> Self::AbsoluteRoute {
        CrateRoute::Api(ApiRoute::Subscriptions(route))
    }
}

impl From<String> for PriceSubscription {
    fn from(market_pair: String) -> Self {
        Self { market_pair }
    }
}
