pub mod subscription_cache;

use crate::{
    database::Schema,
    shared::{
        PriceSubscription,
        PriceHistoryRequest,
        ServerMessage,
        subscription::{
            Request,
            Response,
        },
    },
    websocket::Session,
};
use database_table::{
    Database,
    Entry,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
    stream,
};
use futures::stream::{
    StreamExt,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
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
    StreamHandler,
    AsyncContext,
    Context,
    Addr,
    ResponseActFuture,
};
use actix_interop::{
    FutureInterop,
    with_ctx,
};
use rql::*;
use std::result::Result;
use database_table::DatabaseTable;

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
#[derive(Debug)]
pub struct Subscriptions {
    session: Addr<Session>,
}
impl Subscriptions {
    pub fn init(session: Addr<Session>) -> Addr<Self> {
        Self::create(move |_| Self { session })
    }
    pub async fn add_subscription(req: PriceHistoryRequest) -> Result<Id<PriceSubscription>, Error> {
        caches_mut()
            .await
            .add_subscription(req)
            .await
    }
    pub async fn find_subscription(request: PriceHistoryRequest) -> Option<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>)> {
        caches()
            .await
            .find_subscription(request)
            .await
    }
    pub async fn get_subscription(id: Id<PriceSubscription>) -> Result<Arc<RwLock<SubscriptionCache>>, crate::Error> {
        caches()
            .await
            .get_subscription(id)
            .await
    }
    pub async fn get_subscription_list() -> Vec<Entry<PriceSubscription>> {

        caches()
            .await
            .get_subscription_list()
            .await
    }
    pub async fn update() -> Result<(), Error> {
        caches_mut().await.update().await
    }
}
impl Handler<Request> for Subscriptions {
    type Result = ResponseActFuture<Self, Option<Response>>;
    fn handle(
        &mut self,
        msg: Request,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        async move {
            match msg {
                Request::AddPriceSubscription(request) => {
                    info!("Subscribing to market pair {}", &request.market_pair);
                    let id = Self::add_subscription(request.clone()).await.unwrap();
                    // TODO interval/timer handles
                    //crate::server::interval::set(interval(Duration::from_secs(1)));
                    Some(Response::SubscriptionAdded(id))
                },
                Request::GetPriceSubscriptionList => {
                    info!("Getting subscription list");
                    let list = Self::get_subscription_list().await;
                    Some(Response::SubscriptionList(list))
                },
                Request::GetHistoryUpdates(id) => {
                    info!("Starting history updates of subscription {:#?}", id);
                    let sub = Self::get_subscription(id).await.unwrap();
                    let sub = sub.read().await;
                    with_ctx::<Self, _, _>(|_act, ctx| {
                        ctx.add_stream(stream::interval(sub.time_interval.to_duration().to_std().unwrap())
                            .map(move |_| Request::SendPriceHistory(id.clone()))
                        );
                    });
                    let sub = Self::get_subscription(id).await.unwrap();
                    let history = sub.read().await.latest_price_history().await.unwrap();
                    Some(Response::PriceHistory(id, history))
                },
                Request::SendPriceHistory(id) => {
                    info!("Sending price history for {:#?}", id);
                    let sub = Self::get_subscription(id).await.unwrap();
                    let history = sub.read().await.latest_price_history().await.unwrap();
                    with_ctx::<Self, _, _>(|_act, ctx| {
                        ctx.notify(Response::PriceHistory(id, history));
                    });
                    None
                },
            }
        }.interop_actor_boxed(self)
    }
}
impl Handler<Response> for Subscriptions {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Response,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let session = self.session.clone();
        async move {
            session.do_send(ServerMessage::Subscriptions(msg));
        }.interop_actor_boxed(self)
    }
}
impl StreamHandler<Request> for Subscriptions {
    fn handle(
        &mut self,
        msg: Request,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
impl Actor for Subscriptions {
    type Context = Context<Self>;
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
#[derive(Default)]
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
    pub async fn add_subscription(&mut self, request: PriceHistoryRequest) -> Result<Id<PriceSubscription>, Error> {
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
        request: PriceHistoryRequest,
    ) -> Option<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>)> {
        let req = Arc::new(request);
        futures::stream::iter(self.subscriptions.iter())
            .then(move |(id, cache)| {
                let req = req.clone(); 
                async move {
                    if cache.read().await.subscription == req.as_ref() {
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
    pub async fn get_subscription<'a>(
        &'a self,
        id: Id<PriceSubscription>,
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
