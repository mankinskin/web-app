use shared::{
    subscriptions::{
        PriceSubscription,
        SubscriptionRequest,
    },
};
use crate::{
    websocket::Session,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};
use rql::*;
use riker::actors::*;

#[actor(Msg)]
#[derive(Debug)]
pub struct SubscriptionCacheActor {
    id: Id<PriceSubscription>,
    session: ActorRef<<Session as Actor>::Msg>,
    //update_stream: Option<SpawnHandle>,
}
impl ActorFactoryArgs<(Id<PriceSubscription>, ActorRef<<Session as Actor>::Msg>)> for SubscriptionCacheActor {
    fn create_args((id, session): (Id<PriceSubscription>, ActorRef<<Session as Actor>::Msg>)) -> Self {
        info!("Creating SubscriptionCacheActor");
        Self {
            id,
            session,
        }
    }
}
impl Actor for SubscriptionCacheActor {
    type Msg = SubscriptionCacheActorMsg;
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}
#[derive(Debug, Clone)]
pub enum Msg {
    Request(SubscriptionRequest),
    Refresh,
}
impl Receive<Msg> for SubscriptionCacheActor {
    type Msg = SubscriptionCacheActorMsg;
    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Msg, sender: Sender) {
        let id = self.id.clone();
        ctx.run(async move {
            //match msg {
            //    Msg::Request(req) =>
            //        match req {
            //            SubscriptionRequest::UpdatePriceSubscription(request) => {
            //                info!("Updating subscription {}", &id);
            //                Self::update_subscription(id, request.clone()).await.unwrap();
            //                Some(Response::SubscriptionUpdated)
            //            },
            //            SubscriptionRequest::StartHistoryUpdates => {
            //                info!("Starting history updates of subscription {:#?}", id);
            //                with_ctx::<Self, _, _>(|act, ctx| {
            //                    act.update_stream = Some(ctx.add_stream(
            //                        stream::interval(std::time::Duration::from_secs(3))
            //                            .map(move |_| Msg::Refresh)
            //                    ));
            //                    ctx.notify(Msg::Refresh);
            //                });
            //                None
            //            },
            //        },
            //    Msg::Refresh => {
            //        //info!("Updating price history for {:#?}", id);
            //        let sub = Self::get_subscription(id).await.unwrap();
            //        let mut sub = sub.write().await;
            //        sub.refresh().await.unwrap();
            //        if let Some(history) = sub.get_new_history().await {
            //            with_ctx::<Self, _, _>(|_act, ctx| {
            //                ctx.notify(Response::PriceHistory(id.clone(), history));
            //            });
            //        }
            //        None
            //    },
            //}
        });
    }
}
