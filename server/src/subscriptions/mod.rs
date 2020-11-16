pub mod cache;

#[cfg(feature = "actix_server")]
pub mod actix_actor;
#[cfg(feature = "actix_server")]
pub use actix_actor as actor;

#[cfg(not(feature = "actix_server"))]
pub mod riker_actor;
#[cfg(not(feature = "actix_server"))]
pub use riker_actor as actor;

pub use {
    actor::SubscriptionsActor,
    cache::actor::SubscriptionCacheActor,
};
use crate::{
    database::Schema,
};
use shared::{
    subscriptions::{
        PriceSubscription,
        UpdatePriceSubscriptionRequest,
    },
};
use cache::SubscriptionCache;
use database_table::{
    Database,
    Entry,
    DatabaseTable,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};
use std::{
    result::Result,
    fmt::{
        Formatter,
        Display,
        self,
    },
    collections::HashMap,
};
use futures::stream::{
    StreamExt,
};
use lazy_static::lazy_static;
#[allow(unused)]
use tracing::{
    debug,
    info,
};
use rql::*;

#[derive(Clone, Debug)]
pub enum Error {
    Text(String),
    Binance(crate::binance::Error),
    Multiple(Vec<Error>),
}
impl From<crate::binance::Error> for Error {
    fn from(err: crate::binance::Error) -> Self {
        Self::Binance(err)
    }
}
impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Text(err)
    }
}
impl From<Vec<Error>> for Error {
    fn from(errs: Vec<Error>) -> Self {
        Self::Multiple(errs)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Multiple(v) => write!(f, "{:#?}", v),
            Self::Binance(e) => write!(f, "{:#?}", e),
        }
    }
}

lazy_static! {
    static ref CACHES: Arc<RwLock<StaticSubscriptions>> = Arc::new(RwLock::new(StaticSubscriptions::new()));
}
pub async fn caches() -> RwLockReadGuard<'static, StaticSubscriptions> {
    CACHES.read().await
}
pub async fn caches_mut() -> RwLockWriteGuard<'static, StaticSubscriptions> {
    CACHES.write().await
}
pub async fn add_subscription(req: PriceSubscription) -> Result<Id<PriceSubscription>, Error> {
    caches_mut()
        .await
        .add_subscription(req)
        .await
}
pub async fn update_subscription(id: Id<PriceSubscription>, req: UpdatePriceSubscriptionRequest) -> Result<(), Error> {
    caches_mut()
        .await
        .update_subscription(id, req)
        .await
}
pub async fn find_subscription(request: PriceSubscription) -> Option<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>)> {
    caches()
        .await
        .find_subscription(request)
        .await
}
pub async fn get_subscription(id: Id<PriceSubscription>) -> Result<Arc<RwLock<SubscriptionCache>>, Error> {
    caches()
        .await
        .get_subscription(id)
        .await
}
pub async fn delete_subscription(id: Id<PriceSubscription>) -> Result<(), Error> {
    caches_mut()
        .await
        .delete_subscription(id)
        .await
}
pub async fn get_subscription_list() -> Vec<Entry<PriceSubscription>> {

    caches()
        .await
        .get_subscription_list()
        .await
}
pub async fn refresh() -> Result<(), Error> {
    caches_mut().await.refresh().await
}

#[derive(Debug)]
pub struct StaticSubscriptions {
    pub subscriptions: HashMap<Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>>,
    new_subscriptions: bool,
}
impl StaticSubscriptions {
    fn load_subscriptions_table() -> HashMap<Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>> {
        <PriceSubscription as DatabaseTable<'_, Schema>>::table()
            .rows()
            .map(|row| (row.id.clone(), Arc::new(RwLock::new(SubscriptionCache::from(row.data.clone())))))
            .collect()
    }
    pub fn new() -> Self {
        let subscriptions = Self::load_subscriptions_table();
        Self {
            subscriptions,
            new_subscriptions: false,
        }
    }
    pub async fn update_subscription(&mut self, id: Id<PriceSubscription>, request: UpdatePriceSubscriptionRequest) -> Result<(), Error> {
        debug!("Updating subscription...");
        self.get_subscription(id).await?
            .write().await
            .update(request).await
    }
    pub async fn add_subscription(&mut self, request: PriceSubscription) -> Result<Id<PriceSubscription>, Error> {
        debug!("Adding subscription...");
        if let Some((id, _)) = self.find_subscription(request.clone()).await {
            debug!("Model already exists.");
            Ok(id)
        } else {
            let sub = PriceSubscription::from(request);
            let id = DatabaseTable::<'_, Schema>::table_mut()
                .insert(sub.clone());
            self.subscriptions.insert(id.clone(), Arc::new(RwLock::new(SubscriptionCache::from(sub))));
            self.new_subscriptions = true;
            Ok(id)
        }
    }
    pub async fn find_subscription<'a>(
        &'a self,
        request: PriceSubscription,
    ) -> Option<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>)> {
        let req = Arc::new(request);
        futures::stream::iter(self.subscriptions.iter())
            .then(move |(id, cache)| {
                let req = req.clone(); 
                async move {
                    if cache.read().await.market_pair == req.as_ref().market_pair {
                        Some((id.clone(), cache.clone()))
                    } else {
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .find_map(|opt| opt)
    }
    pub async fn delete_subscription<'a>(
        &'a mut self,
        id: Id<PriceSubscription>,
    ) -> Result<(), Error> {
        self.subscriptions
            .remove(&id)
            .map(|_| ())
            .ok_or(Error::from(format!(
                "No Subscription with ID: {}",
                id
            )))
    }
    pub async fn get_subscription<'a>(
        &'a self,
        id: Id<PriceSubscription>,
    ) -> Result<Arc<RwLock<SubscriptionCache>>, Error> {
        self.subscriptions
            .get(&id)
            .map(Clone::clone)
            .ok_or(Error::from(format!(
                "No Subscription with ID: {}",
                id
            )))
    }
    pub async fn filter_available_symbols(&mut self) -> Result<(), Error> {
        let mut errors = Vec::new();
        self.subscriptions = futures::stream::iter(self.subscriptions.clone().into_iter())
            .then(async move |(id, cache)| {
                if cache.read().await.is_available().await {
                    Ok((id, cache))
                } else {
                    Err(Error::from(format!(
                        "Symbol {} does not exist on binance.",
                        cache.read().await.market_pair
                    )))
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|result: Result<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>), Error>| {
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
            Err(Error::from(errors))
        }
    }
    pub async fn get_subscription_list(&self) -> Vec<Entry<PriceSubscription>> {
        <Schema as Database::<'_, PriceSubscription>>::get_all()
    }

    pub async fn refresh(&mut self) -> Result<(), Error> {
        //debug!("Model update");
        if self.new_subscriptions {
            self.filter_available_symbols().await?;
            self.new_subscriptions = false;
        }
        for (_, cache) in &mut self.subscriptions {
            cache.write().await.refresh().await?
        }
        Ok(())
    }
}
