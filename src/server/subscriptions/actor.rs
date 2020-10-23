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
    subscriptions::{
        caches,
        caches_mut,
        Error,
        cache::{
            SubscriptionCache,
            actor::SubscriptionCacheActor,
        },
    },
};
use database_table::{
    Entry,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
    },
};
#[allow(unused)]
use tracing::{
    debug,
    info,
};
use std::{
    result::Result,
    collections::HashMap,
};
use actix::{
    Actor,
    Handler,
    StreamHandler,
    AsyncContext,
    Context,
    Addr,
    ResponseActFuture,
    Message,
};
use actix_interop::{
    FutureInterop,
    with_ctx,
};
use rql::*;

#[derive(Debug)]
pub struct SubscriptionsActor {
    session: Addr<Session>,
    actors: HashMap<Id<PriceSubscription>, Addr<SubscriptionCacheActor>>,
}
impl Actor for SubscriptionsActor {
    type Context = Context<Self>;
}
impl SubscriptionsActor {
    pub async fn init(session: Addr<Session>) -> Addr<Self> {
        info!("Creating SubscriptionsActor");
        let actors = caches().await
            .subscriptions
            .iter()
            .map(|(id, _)|
                (
                    id.clone(), 
                    SubscriptionCacheActor::init(id.clone(), session.clone())
                )
            )
            .collect();
        Self::create(move |_| Self {
            session,
            actors,
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
}
impl Handler<Msg> for SubscriptionsActor {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Msg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        async move {
            match msg {
            }
        }.interop_actor_boxed(self)
    }
}
impl Handler<Request> for SubscriptionsActor {
    type Result = ResponseActFuture<Self, Option<Response>>;
    fn handle(
        &mut self,
        msg: Request,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let session = self.session.clone();
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
                    with_ctx::<Self, _, _>(|act, _ctx| {
                        act.actors.insert(id.clone(), SubscriptionCacheActor::init(id.clone(), session));
                    });
                    Some(Response::SubscriptionAdded(id))
                },
                Request::Subscription(id, req) => {
                    let id = id.clone();
                    info!("Request for Subscription {:#?}", id);
                    let addr = with_ctx::<Self, _, _>(move |act, _ctx| {
                        act.actors.get(&id).map(Clone::clone)
                    });
                    if let Some(sub) = addr {
                        sub.send(req.clone()).await.unwrap()
                    } else {
                        info!("Subscription {:#?} not found", id);
                        Some(Response::SubscriptionNotFound(id))
                    }
                }
            }
        }.interop_actor_boxed(self)
    }
}
impl Handler<Response> for SubscriptionsActor {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Response,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let session = self.session.clone();
        async move {
            session.send(ServerMessage::Subscriptions(msg)).await.unwrap();
        }.interop_actor_boxed(self)
    }
}
impl StreamHandler<Request> for SubscriptionsActor {
    fn handle(
        &mut self,
        msg: Request,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
impl StreamHandler<Msg> for SubscriptionsActor {
    fn handle(
        &mut self,
        msg: Msg,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
