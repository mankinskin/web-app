pub mod subscription_cache;
pub mod subscriptions;

use crate::{
    shared::{
        ServerMessage,
        subscription::{
            Request,
            Response,
            PriceSubscription,
            PriceSubscriptionRequest,
        },
    },
    websocket::Session,
};
use database_table::{
    Entry,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
    },
    stream,
};
use futures::stream::{
    StreamExt,
};
#[allow(unused)]
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
    SpawnHandle,
    Message,
};
use actix_interop::{
    FutureInterop,
    with_ctx,
};
use rql::*;
use std::result::Result;
use subscriptions::{
    caches,
    caches_mut,
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
#[derive(Debug)]
pub struct Subscriptions {
    session: Addr<Session>,
    update_stream: Option<SpawnHandle>,
}
impl Subscriptions {
    pub fn init(session: Addr<Session>) -> Addr<Self> {
        Self::create(move |_| Self {
            session,
            update_stream: None,
        })
    }
    pub async fn add_subscription(req: PriceSubscriptionRequest) -> Result<Id<PriceSubscription>, Error> {
        caches_mut()
            .await
            .add_subscription(req)
            .await
    }
    pub async fn update_subscription(req: PriceSubscriptionRequest) -> Result<(), Error> {
        caches_mut()
            .await
            .update_subscription(req)
            .await
    }
    pub async fn find_subscription(request: PriceSubscriptionRequest) -> Option<(Id<PriceSubscription>, Arc<RwLock<SubscriptionCache>>)> {
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

#[derive(Message)]
#[rtype(result = "()")]
enum Msg {
    UpdateSubscription(Id<PriceSubscription>),
    SendPriceHistory(Id<PriceSubscription>),
}
impl Handler<Msg> for Subscriptions {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Msg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        async move {
            match msg {
                Msg::UpdateSubscription(id) => {
                    info!("Updating price history {:#?}", id);
                },
                Msg::SendPriceHistory(id) => {
                    info!("Sending price history for {:#?}", id);
                    let sub = Self::get_subscription(id).await.unwrap();
                    let history = sub.read().await.get_latest_price_history().await.unwrap();
                    with_ctx::<Self, _, _>(|_act, ctx| {
                        ctx.notify(Response::PriceHistory(id, history));
                    });
                },
            }
        }.interop_actor_boxed(self)
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
                Request::GetPriceSubscriptionList => {
                    info!("Getting subscription list");
                    let list = Self::get_subscription_list().await;
                    Some(Response::SubscriptionList(list))
                },
                Request::AddPriceSubscription(request) => {
                    info!("Subscribing to market pair {}", &request.market_pair);
                    let id = Self::add_subscription(request.clone()).await.unwrap();
                    Some(Response::SubscriptionAdded(id))
                },
                Request::UpdatePriceSubscription(request) => {
                    info!("Updating subscription {}", &request.market_pair);
                    Self::update_subscription(request.clone()).await.unwrap();
                    Some(Response::SubscriptionUpdated)
                },
                Request::StartHistoryUpdates(id) => {
                    info!("Starting history updates of subscription {:#?}", id);
                    with_ctx::<Self, _, _>(|act, ctx| {
                        act.update_stream = Some(ctx.add_stream(stream::interval(std::time::Duration::from_secs(3))
                            .map(move |_| Msg::SendPriceHistory(id.clone()))
                        ));
                        ctx.notify(Msg::SendPriceHistory(id.clone()));
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
impl StreamHandler<Msg> for Subscriptions {
    fn handle(
        &mut self,
        msg: Msg,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
impl Actor for Subscriptions {
    type Context = Context<Self>;
}
