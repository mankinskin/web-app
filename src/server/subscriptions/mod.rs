pub mod subscription_cache;

use async_std::sync::MutexGuard;
use crate::shared::PriceHistoryRequest;
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
use subscription_cache::SubscriptionCache;

#[derive(Debug)]
pub struct Error(String);

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s)
    }
}
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
lazy_static! {
    pub static ref SUBSCRIPTIONS: Arc<Mutex<Subscriptions>> = Arc::new(Mutex::new(Subscriptions::new()));
    static ref SUBSCRIPTION_IDS: AtomicUsize = AtomicUsize::new(0);
}
pub async fn subscriptions() -> MutexGuard<'static, Subscriptions> {
    SUBSCRIPTIONS.lock().await
}
#[derive(Default)]
pub struct Subscriptions {
    pub subscriptions: HashMap<usize, SubscriptionCache>,
    new_subscriptions: bool,
}
impl Subscriptions {
    pub fn new() -> Self {
        Self::default()
    }
    fn new_subscription_id() -> usize {
        SUBSCRIPTION_IDS.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn add_subscription(&mut self, request: PriceHistoryRequest) -> Result<usize, crate::Error> {
        debug!("Adding subscription...");
        if let Some(id) = self.subscriptions
            .iter()
            .find_map(|(id, cache)| if cache.subscription == request { Some(id) } else { None })
        {
            debug!("Model already exists.");
            Ok(id.clone())
        } else {
            let id = Self::new_subscription_id();
            self.subscriptions.insert(id.clone(), SubscriptionCache::from(request));
            self.new_subscriptions = true;
            Ok(id)
        }
    }
    pub async fn get_subscription<'a>(
        &'a self,
        id: usize,
    ) -> Result<&'a SubscriptionCache, crate::Error> {
        self.subscriptions
            .get(&id)
            .ok_or(crate::Error::from(Error::from(format!(
                "No Subscription with ID: {}",
                id
            ))))
    }
    pub async fn filter_available_symbols(&mut self) -> Result<(), crate::Error> {
        let mut errors = Vec::new();
        self.subscriptions = futures::stream::iter(self.subscriptions.clone().into_iter())
            .then(async move |(id, cache)| {
                if cache.subscription.is_available().await {
                    Ok((id, cache))
                } else {
                    Err(Error::from(format!(
                        "Symbol {} does not exist on binance.",
                        cache.subscription.market_pair
                    )))
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|result: Result<(usize, SubscriptionCache), Error>| {
                match result {
                    Ok(pair) => Some(pair),
                    Err(error) => {
                        errors.push(error);
                        None
                    }
                }
            })
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::from(errors))
        }
    }
    pub async fn update(&mut self) -> Result<(), crate::Error> {
        //debug!("Model update");
        if self.new_subscriptions {
            self.filter_available_symbols().await?;
            self.new_subscriptions = false;
        }
        for (_, cache) in &mut self.subscriptions {
            cache.update().await?
        }
        Ok(())
    }
}

