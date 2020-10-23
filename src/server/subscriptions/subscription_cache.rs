use crate::{
    shared::subscription::{
        PriceSubscription,
        PriceSubscriptionRequest,
    },
    binance::{
        Binance,
        PriceHistoryRequest,
    },
};
use app_model::market::{
    PriceHistory,
};
use openlimits::model::{
    Interval,
    Candle,
    Paginator,
};
use chrono::{
    DateTime,
    Utc,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};
use super::Error;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SubscriptionCache {
    pub market_pair: String,
    pub interval: Interval,
    pub last_update: Option<DateTime<Utc>>,
    prices: HashMap<Interval, Vec<Candle>>,
}
impl From<PriceSubscription> for SubscriptionCache {
    fn from(sub: PriceSubscription) -> Self {
        let interval = sub.interval.unwrap_or(Interval::OneMinute);
        let mut prices = HashMap::new();
        prices.insert(interval, Vec::new());
        Self {
            market_pair: sub.market_pair,
            prices,
            interval,
            last_update: None,
        }
    }
}
impl From<PriceSubscriptionRequest> for SubscriptionCache {
    fn from(request: PriceSubscriptionRequest) -> Self {
        Self::from(PriceSubscription::from(request))
    }
}
impl SubscriptionCache {
    pub async fn process(&mut self, req: PriceSubscriptionRequest) -> Result<(), Error> {
        debug!["Processing update request"];
        if let Some(interval) = req.interval {
            debug!["setting interval: {:?}", interval];
            self.interval = interval;
        }
        Ok(())
    }
    pub async fn update(&mut self) -> Result<(), Error> {
        //let candles = self.latest_price_history().await?.candles;
        //self.prices.extend(candles.into_iter());
        Ok(())
    }
    pub async fn paginator(&self) -> Option<Paginator<u32>> {
        self.last_update.map(|datetime| {
            Paginator {
                start_time: Some(datetime.timestamp_millis() as u64),
                ..Default::default()
            }
        })
    }
    pub async fn is_available(&self) -> bool {
        let symbol = self.market_pair.to_uppercase();
        crate::binance::Binance::symbol_available(&symbol).await
    }
    async fn price_history_request(&self) -> PriceHistoryRequest {
        let paginator = self.paginator().await;
        PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.interval),
            paginator: paginator,
        }
    }
    pub async fn get_latest_price_history(&self) -> Result<PriceHistory, crate::binance::Error> {
        let req = self.price_history_request().await;
        Binance::get_symbol_price_history(req).await.map_err(|e| e.to_string().into())
    }
}
