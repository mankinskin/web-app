use crate::{
    shared::{
        PriceHistoryRequest,
        PriceSubscription,
    },
};
use openlimits::model::Candle;
use serde::{
    Deserialize,
    Serialize,
};
use super::Error;
use std::ops::Deref;

#[derive(Serialize, Deserialize, Clone)]
pub struct SubscriptionCache {
    pub subscription: PriceSubscription,
    prices: Vec<Candle>,
}
impl From<PriceSubscription> for SubscriptionCache {
    fn from(sub: PriceSubscription) -> Self {
        Self {
            subscription: sub,
            prices: Vec::new(),
        }
    }
}
impl From<PriceHistoryRequest> for SubscriptionCache {
    fn from(request: PriceHistoryRequest) -> Self {
        Self::from(PriceSubscription::from(request))
    }
}
impl SubscriptionCache {
    pub async fn update(&mut self) -> Result<(), Error> {
        //debug!("SymbolModel update");
        let candles = self.subscription.latest_price_history().await?.candles;
        self.prices.extend(candles.into_iter());
        Ok(())
    }
}
impl Deref for SubscriptionCache {
    type Target = PriceSubscription;
    fn deref(&self) -> &Self::Target {
        &self.subscription
    }
}
