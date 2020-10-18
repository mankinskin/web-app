pub mod subscription_cache;

use crate::shared::{
    PriceSubscription,
    PriceHistoryRequest,
    ClientMessage,
    ServerMessage,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};
use futures::stream::{
    Stream,
    StreamExt,
};
use chrono::{
    DateTime,
    Utc,
};
use lazy_static::lazy_static;
use openlimits::model::Candle;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::convert::TryInto;
use tracing::{
    debug,
    info,
};
use subscription_cache::SubscriptionCache;
use std::fmt::{
    Formatter,
    Display,
    self,
};
use actix::{
    Actor,
    Handler,
    Context,
    Addr,
    ResponseActFuture,
};
use actix_interop::{
    FutureInterop,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

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
pub struct Subscriptions;
impl Subscriptions {
    pub async fn init() -> Addr<Self> {
        Self::create(move |_| Self)
    }
    pub async fn add_subscription(req: PriceHistoryRequest) -> Result<usize, Error> {
        subscriptions_mut()
            .await
            .add_subscription(req)
            .await
    }
    pub async fn get_subscription(id: usize) -> Result<Arc<RwLock<SubscriptionCache>>, crate::Error> {
        subscriptions()
            .await
            .get_subscription(id)
            .await
            .clone()
    }
    pub async fn get_subscription_list() -> Result<HashMap<usize, PriceSubscription>, crate::Error> {
        crate::subscriptions()
            .await
            .get_subscription_list()
            .await
    }
    pub async fn update() -> Result<(), Error> {
        subscriptions_mut().await.update().await
    }
}
impl Handler<ClientMessage> for Subscriptions {
    type Result = ResponseActFuture<Self, Option<ServerMessage>>;
    fn handle(
        &mut self,
        msg: ClientMessage,
        _: &mut Self::Context,
    ) -> Self::Result {
        async move {
            match msg {
                ClientMessage::AddPriceSubscription(request) => {
                    info!("Subscribing to market pair {}", &request.market_pair);
                    let id = Self::add_subscription(request.clone()).await.unwrap();
                    // TODO interval/timer handles
                    //crate::server::interval::set(interval(Duration::from_secs(1)));
                    Some(ServerMessage::SubscriptionAdded(id))
                },
                ClientMessage::GetPriceSubscriptionList => {
                    info!("Getting subscription list");
                    let list = Self::get_subscription_list().await.unwrap();
                    //self.get_symbol_price_history(request.clone()).await
                    //crate::server::interval::set(interval(Duration::from_secs(1)));
                    Some(ServerMessage::SubscriptionList(list))
                },
                _ => None,
            }
        }.interop_actor_boxed(self)
    }
}
impl Actor for Subscriptions {
    type Context = Context<Self>;
}

lazy_static! {
    static ref SUBSCRIPTIONS: Arc<RwLock<StaticSubscriptions>> = Arc::new(RwLock::new(StaticSubscriptions::new()));
    static ref SUBSCRIPTION_IDS: AtomicUsize = AtomicUsize::new(0);
}
pub async fn subscriptions() -> RwLockReadGuard<'static, StaticSubscriptions> {
    SUBSCRIPTIONS.read().await
}
pub async fn subscriptions_mut() -> RwLockWriteGuard<'static, StaticSubscriptions> {
    SUBSCRIPTIONS.write().await
}
#[derive(Default)]
pub struct StaticSubscriptions {
    pub subscriptions: HashMap<usize, Arc<RwLock<SubscriptionCache>>>,
    new_subscriptions: bool,
}
impl StaticSubscriptions {
    pub fn new() -> Self {
        Self::default()
    }
    fn new_subscription_id() -> usize {
        SUBSCRIPTION_IDS.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn add_subscription(&mut self, request: PriceHistoryRequest) -> Result<usize, Error> {
        debug!("Adding subscription...");
        let req = Arc::new(request.clone());
        if let Some(id) = futures::stream::iter(self.subscriptions.iter())
            .then(move |(id, cache)| {
                let req = req.clone(); 
                async move {
                    if cache.read().await.subscription == req.as_ref() {
                        Some(id)
                    } else {
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .find_map(|opt| opt)
        {
            debug!("Model already exists.");
            Ok(id.clone())
        } else {
            let id = Self::new_subscription_id();
            self.subscriptions.insert(id.clone(), Arc::new(RwLock::new(SubscriptionCache::from(request))));
            self.new_subscriptions = true;
            Ok(id)
        }
    }
    pub async fn get_subscription<'a>(
        &'a self,
        id: usize,
    ) -> Result<Arc<RwLock<SubscriptionCache>>, crate::Error> {
        self.subscriptions
            .get(&id)
            .map(Clone::clone)
            .ok_or(crate::Error::from(Error::from(format!(
                "No Subscription with ID: {}",
                id
            ))))
    }
    pub async fn filter_available_symbols(&mut self) -> Result<(), Error> {
        let mut errors = Vec::new();
        self.subscriptions = futures::stream::iter(self.subscriptions.clone().into_iter())
            .then(async move |(id, cache)| {
                if cache.read().await.subscription.is_available().await {
                    Ok((id, cache))
                } else {
                    Err(Error::from(format!(
                        "Symbol {} does not exist on binance.",
                        cache.read().await.subscription.market_pair
                    )))
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|result: Result<(usize, Arc<RwLock<SubscriptionCache>>), Error>| {
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
    pub async fn get_subscription_list(&self) -> Result<HashMap<usize, PriceSubscription>, crate::Error> {
        Ok(futures::stream::iter(self.subscriptions.iter())
            .then(async move |(id, cache)| (id.clone(), cache.read().await.subscription.clone()))
            .collect()
            .await)
    }
    pub async fn update(&mut self) -> Result<(), Error> {
        //debug!("Model update");
        if self.new_subscriptions {
            self.filter_available_symbols().await?;
            self.new_subscriptions = false;
        }
        for (_, cache) in &mut self.subscriptions {
            cache.write().await.update().await?
        }
        Ok(())
    }
}
