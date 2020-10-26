use serde::{
    Deserialize,
    Serialize,
};

#[cfg(target_arch = "wasm32")]
use {
    crate::shared::subscriptions::Route,
    database_table::{
        RemoteTable,
        TableRoutable,
        Entry,
    },
    components::{
        Component,
        Edit,
    },
    seed::{
        *,
        browser::fetch::{
            fetch,
            Request,
            Method,
        },
        prelude::*,
    },
    async_trait::async_trait,
    enum_paths::AsPath,
    std::result::Result,
    rql::*,
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
#[async_trait(?Send)]
impl RemoteTable for PriceSubscription {
    type Error = FetchError;
    async fn get(id: Id<Self>) -> Result<Option<Entry<Self>>, Self::Error> {
        fetch(
            Request::new(Self::entry_route(id).as_path())
                .method(Method::Get)
        ).await?
        .json().await
    }
    async fn delete(id: Id<Self>) -> Result<Option<Self>, Self::Error> {
        fetch(
            Request::new(Self::entry_route(id).as_path())
                .method(Method::Delete)
        ).await?
        .json().await
    }
    async fn get_all() -> Result<Vec<Entry<Self>>, Self::Error> {
        fetch(
            Request::new(Self::table_route().as_path())
                .method(Method::Get)
        ).await?
        .json().await
    }
    async fn post(data: Self) -> Result<Id<Self>, Self::Error> {
        fetch(
            Request::new(Self::table_route().as_path())
                .method(Method::Post)
                .json(&data)?
        ).await?
        .json().await
    }
}

#[cfg(target_arch = "wasm32")]
impl TableRoutable for PriceSubscription {
    type Route = Route;
    fn table_route() -> Self::Route {
        Route::List
    }
    fn entry_route(id: Id<Self>) -> Self::Route {
        Route::Entry(id)
    }
}


impl From<String> for PriceSubscription {
    fn from(market_pair: String) -> Self {
        Self {
            market_pair,
        }
    }
}
