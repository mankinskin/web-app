use crate::{
    shared::{
        subscription::{
            PriceSubscription,
            PriceSubscriptionRequest,
            SubscriptionRequest,
            Response,
        },
        ServerMessage,
    },
    binance::{
        Binance,
        PriceHistoryRequest,
    },
    websocket::Session,
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
use rql::*;
use actix::{
    Actor,
    Handler,
    StreamHandler,
    AsyncContext,
    Context,
    Addr,
    ResponseActFuture,
    SpawnHandle,
    Message,
};
use actix_interop::{
    FutureInterop,
    with_ctx,
};
use async_std::{
    stream,
};
use futures::stream::{
    StreamExt,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
    },
};
use super::subscriptions::{
    caches,
    caches_mut,
};
#[derive(Debug)]
pub struct SubscriptionCacheActor {
    id: Id<PriceSubscription>,
    session: Addr<Session>,
    update_stream: Option<SpawnHandle>,
}
impl SubscriptionCacheActor {
    pub fn init(id: Id<PriceSubscription>, session: Addr<Session>) -> Addr<Self> {
        info!("Creating SubscriptionCacheActor {:#?}", id);
        Self::create(|_| Self {
            id,
            session,
            update_stream: None,
        })
    }
    pub async fn get_subscription(id: Id<PriceSubscription>) -> Result<Arc<RwLock<SubscriptionCache>>, Error> {
        caches()
            .await
            .get_subscription(id)
            .await
    }
    pub async fn update_subscription(req: PriceSubscriptionRequest) -> Result<(), Error> {
        caches_mut()
            .await
            .update_subscription(req)
            .await
    }
}
impl Actor for SubscriptionCacheActor {
    type Context = Context<Self>;
}
#[derive(Message)]
#[rtype(result = "Option<Response>")]
enum Msg {
    Request(SubscriptionRequest),
    SendPriceHistory,
}
impl Handler<Msg> for SubscriptionCacheActor {
    type Result = ResponseActFuture<Self, Option<Response>>;
    fn handle(
        &mut self,
        msg: Msg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let id = self.id.clone();
        async move {
            match msg {
                Msg::Request(req) =>
                    match req {
                        SubscriptionRequest::UpdatePriceSubscription(request) => {
                            info!("Updating subscription {}", &request.market_pair);
                            Self::update_subscription(request.clone()).await.unwrap();
                            Some(Response::SubscriptionUpdated)
                        },
                        SubscriptionRequest::StartHistoryUpdates => {
                            info!("Starting history updates of subscription {:#?}", id);
                            with_ctx::<Self, _, _>(|act, ctx| {
                                act.update_stream = Some(ctx.add_stream(stream::interval(std::time::Duration::from_secs(3))
                                    .map(move |_| Msg::SendPriceHistory)
                                ));
                                ctx.notify(Msg::SendPriceHistory);
                            });
                            None
                        },
                    },
                Msg::SendPriceHistory => {
                    info!("Sending price history for {:#?}", id);
                    let sub = Self::get_subscription(id).await.unwrap();
                    let history = sub.read().await.get_latest_price_history().await.unwrap();
                    with_ctx::<Self, _, _>(|_act, ctx| {
                        ctx.notify(Response::PriceHistory(id.clone(), history));
                    });
                    None
                },
            }
        }.interop_actor_boxed(self)
    }
}
impl Handler<SubscriptionRequest> for SubscriptionCacheActor {
    type Result = ResponseActFuture<Self, Option<Response>>;
    fn handle(
        &mut self,
        msg: SubscriptionRequest,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let addr = ctx.address().clone();
        async move {
            addr.send(Msg::Request(msg)).await.unwrap()
        }.interop_actor_boxed(self)
    }
}
impl StreamHandler<Msg> for SubscriptionCacheActor {
    fn handle(
        &mut self,
        msg: Msg,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
impl Handler<Response> for SubscriptionCacheActor {
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
