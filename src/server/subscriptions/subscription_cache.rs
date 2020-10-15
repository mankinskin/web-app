use async_std::sync::MutexGuard;
use crate::{
    shared::{
        PriceHistoryRequest,
        PriceSubscription,
    },
};
use async_std::sync::{
    Arc,
    Mutex,
};
use chrono::{
    DateTime,
    Utc,
};
use futures::StreamExt;
use lazy_static::lazy_static;
use openlimits::model::Candle;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::convert::TryInto;
use tracing::debug;
use super::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct SubscriptionCache {
    pub subscription: PriceSubscription,
    prices: Vec<Candle>,
}
impl From<PriceHistoryRequest> for SubscriptionCache {
    fn from(request: PriceHistoryRequest) -> Self {
        Self {
            subscription: PriceSubscription::from(request),
            prices: Vec::new(),
        }
    }
}
impl SubscriptionCache {
    pub async fn update(&mut self) -> Result<(), Error> {
        //debug!("SymbolModel update");
        let candles = self.subscription.latest_price_candles().await?;
        self.prices.extend(candles.into_iter());
        Ok(())
    }
}
