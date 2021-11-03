use shared::{
	subscriptions::{
		PriceSubscription,
		SubscriptionRequest,
		Response,
	},
	ServerMessage,
};
use crate::{
	websocket::ConnectionActor,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
};
use rql::*;
use riker::actors::*;
use async_std::{
	task::JoinHandle,
	stream::{
		StreamExt,
		interval,
	},
};
use std::{
	sync::Arc,
	time::Duration,
};
#[actor(Msg)]
#[derive(Debug)]
pub struct SubscriptionCacheActor {
	id: Id<PriceSubscription>,
	connection: ActorRef<<ConnectionActor as Actor>::Msg>,
	update_stream: Option<Arc<JoinHandle<()>>>,
}
impl ActorFactoryArgs<(Id<PriceSubscription>, ActorRef<<ConnectionActor as Actor>::Msg>)> for SubscriptionCacheActor {
	fn create_args((id, connection): (Id<PriceSubscription>, ActorRef<<ConnectionActor as Actor>::Msg>)) -> Self {
		info!("Creating SubscriptionCacheActor");
		Self {
			id,
			connection,
			update_stream: None,
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
	SetUpdateStream(Arc<JoinHandle<()>>),
}
impl From<SubscriptionRequest> for Msg {
	fn from(req: SubscriptionRequest) -> Self {
		Self::Request(req)
	}
}
impl Receive<Msg> for SubscriptionCacheActor {
	type Msg = SubscriptionCacheActorMsg;
	fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Msg, sender: Sender) {
		trace!("SubscriptionCacheActor Msg");
		let id = self.id.clone();
		let connection = self.connection.clone();
		let msg2 = msg.clone();
		let myself = ctx.myself();
		ctx.run(async move {
			let msg = msg2;
			match msg {
				Msg::Request(req) =>
				match req {
					SubscriptionRequest::UpdatePriceSubscription(request) => {
						debug!("Updating subscription {}", &id);
						crate::subscriptions::update_subscription(id, request.clone()).await.unwrap();
						connection.tell(ServerMessage::Subscriptions(Response::SubscriptionUpdated), None);
					},
					SubscriptionRequest::StartHistoryUpdates => {
						debug!("Starting history updates of subscription {:#?}", id);
						let myself2 = myself.clone();
						let sender2 = sender.clone();
						let stream = async_std::task::spawn(async move {
							while let Some(msg) = interval(Duration::from_secs(3))
							.map(move |_| Msg::Refresh)
							.next().await {
								myself2.tell(msg, sender2.clone());
							}
						});
						myself.tell(Msg::Refresh, sender.clone());
						myself.tell(Msg::SetUpdateStream(Arc::new(stream)), sender);
					},
				},
				Msg::Refresh => {
					trace!("Updating price history for {:#?}", id);
					let cache = crate::subscriptions::get_subscription_cache(id).await.unwrap();
					let mut cache = cache.write().await;
					cache.refresh().await.unwrap();
					if let Some(history) = cache.get_new_history().await {
						connection.tell(ServerMessage::Subscriptions(Response::PriceHistory(id.clone(), history)), None);
					}
				},
				_ => {}
			}
		}).expect("Failed to run future");
		match msg {
			Msg::SetUpdateStream(s) => {
				self.update_stream = Some(s);
			},
			_ => {}
		}
	}
}
