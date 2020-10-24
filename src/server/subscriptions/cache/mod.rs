pub mod actor;
use crate::{
    shared::{
        subscription::{
            PriceSubscription,
            PriceSubscriptionRequest,
        },
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
use std::{
    collections::HashMap,
    result::Result,
};

#[derive(Debug)]
pub struct SubscriptionCache {
    pub market_pair: String,
    pub current_interval: Interval,
    caches: HashMap<Interval, IntervalCache>,

    new_history_index: Option<usize>,
}
#[derive(Debug)]
pub struct IntervalCache {
    interval: Interval,
    candles: Vec<Candle>,
    pub last_update: Option<DateTime<Utc>>,
}
impl From<Interval> for IntervalCache {
    fn from(interval: Interval) -> Self {
        Self {
            interval,
            candles: Vec::new(),
            last_update: None,
        }
    }
}
impl IntervalCache {
    pub async fn paginator(&self) -> Option<Paginator<u32>> {
        self.last_update.map(|datetime| {
            Paginator {
                start_time: Some(datetime.timestamp_millis() as u64),
                ..Default::default()
            }
        })
    }
}
impl From<PriceSubscription> for SubscriptionCache {
    fn from(sub: PriceSubscription) -> Self {
        let interval = sub.interval.unwrap_or(Interval::OneMinute);
        let mut caches = HashMap::new();
        caches.insert(interval, IntervalCache::from(interval));
        Self {
            market_pair: sub.market_pair,
            caches, 
            current_interval: interval,
            new_history_index: None,
        }
    }
}
impl From<PriceSubscriptionRequest> for SubscriptionCache {
    fn from(request: PriceSubscriptionRequest) -> Self {
        Self::from(PriceSubscription::from(request))
    }
}
impl SubscriptionCache {
    pub async fn set_interval(&mut self, interval: Interval) -> Result<(), Error> {
        debug!["setting interval: {:?}", interval];
        if !self.caches.contains_key(&interval) {
            self.caches.insert(interval, IntervalCache::from(interval));
        }
        self.current_interval = interval;
        self.new_history_index = None;
        Ok(())
    }
    pub async fn update(&mut self) -> Result<(), Error> {
        //let candles = self.latest_price_history().await?.candles;
        //self.prices.extend(candles.into_iter());
        self.update_price_history().await.unwrap();
        Ok(())
    }
    pub async fn paginator(&self) -> Option<Paginator<u32>> {
        self.current_cache().await.paginator().await
    }
    pub async fn is_available(&self) -> bool {
        let symbol = self.market_pair.to_uppercase();
        crate::binance::Binance::symbol_available(&symbol).await
    }
    async fn price_history_request(&self) -> PriceHistoryRequest {
        let paginator = self.paginator().await;
        PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.current_interval),
            paginator: paginator,
        }
    }
    async fn current_cache(&self) -> &IntervalCache {
        self.caches.get(&self.current_interval).unwrap()
    }
    async fn current_cache_mut(&mut self) -> &mut IntervalCache {
        self.caches.get_mut(&self.current_interval).unwrap()
    }
    pub async fn get_latest_price_history(&self) -> Result<PriceHistory, Error> {
        let req = self.price_history_request().await;
        Binance::get_symbol_price_history(req).await.map_err(|e| e.to_string().into())
    }
    pub async fn get_new_history(&mut self) -> Option<PriceHistory> {
        let candles = self.current_cache().await.candles.clone();
        let i =
            if let Some(i) = self.new_history_index {
                if i == candles.len() {
                    // no new history
                    return None;
                }
                i
            } else {
                0
            };
        self.new_history_index = Some(candles.len());
        Some(PriceHistory {
            market_pair: self.market_pair.clone(),
            time_interval: self.current_interval.clone(),
            candles: candles.into_iter().skip(i).collect(),
        })
    }
    pub async fn update_price_history(&mut self) -> Result<(), Error> {
        let history = self.get_latest_price_history().await?;
        if !history.candles.is_empty() {
            let history_cache = self.current_cache_mut().await;
            history_cache.candles.extend(history.candles.into_iter());
            history_cache.last_update = Some(Utc::now());
        }
        Ok(())
    }
}
