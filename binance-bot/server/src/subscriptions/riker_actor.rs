use shared::{
	subscriptions::{
		Request,
		PriceSubscription,
		Response,
	},
	ServerMessage,
};
use crate::{
	websocket::ConnectionActor,
	subscriptions::{
		get_subscription_list,
		add_subscription,
		caches,
		cache::{
			actor::SubscriptionCacheActor,
		},
	},
};
#[allow(unused)]
use tracing::{
	debug,
	info,
	error,
	trace,
};
use std::{
	collections::HashMap,
	result::Result,
};
use rql::*;

use riker::actors::*;

type CacheActorMap = HashMap<Id<PriceSubscription>, ActorRef<<SubscriptionCacheActor as Actor>::Msg>>;
#[actor(Request, Msg)]
#[derive(Debug)]
pub struct SubscriptionsActor {
	connection: ActorRef<<ConnectionActor as Actor>::Msg>,
	actors: CacheActorMap,
}
impl SubscriptionsActor {
	pub fn actor_name(id: usize) -> String {
		format!("{}_subscriptions_actor", ConnectionActor::actor_name(id))
	}
	pub async fn create(id: usize, connection: ActorRef<<ConnectionActor as Actor>::Msg>) -> Result<ActorRef<<Self as Actor>::Msg>, CreateError> {
		crate::actor_sys().await
			.actor_of_args::<SubscriptionsActor, _> (
				&Self::actor_name(id),
				connection,
			)
	}
}
impl ActorFactoryArgs<ActorRef<<ConnectionActor as Actor>::Msg>> for SubscriptionsActor {
	fn create_args(connection: ActorRef<<ConnectionActor as Actor>::Msg>) -> Self {
		info!("Creating SubscriptionsActor");
		Self {
			connection,
			actors: HashMap::new(),
		}
	}
}
impl Actor for SubscriptionsActor {
	type Msg = SubscriptionsActorMsg;
	fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
		let connection = self.connection.clone();
		let myself = ctx.myself();
		ctx.run(async move {
			let actor_sys = crate::actor_sys().await;
			myself.tell(Msg::SetActors(caches().await
				.subscriptions
				.iter()
				.map(|(id, _)|
					(
						id.clone(),
						actor_sys.actor_of_args::<SubscriptionCacheActor, _>(
							&format!("{}_Subscription_{}_cache_actor", connection.name(), id),
							(id.clone(), connection.clone())
						).unwrap()
					)
				)
				.collect()), None);
		}).expect("Failed to run future!");
	}
	fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
		self.receive(ctx, msg, sender);
	}
}
#[derive(Clone, Debug)]
pub enum Msg {
	SetActors(CacheActorMap),
	AddActor(Id<PriceSubscription>),
}
impl Receive<Msg> for SubscriptionsActor {
	type Msg = SubscriptionsActorMsg;
	fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Msg, sender: Sender) {
		match msg {
			Msg::AddActor(id) => {
				self.actors.insert(id.clone(), ctx.actor_of_args::<SubscriptionCacheActor, _>(
						&format!("ConnectionActor_Subscription_{}_cache_actor", id),
						(id.clone(), self.connection.clone())).unwrap());
				self.connection.tell(ServerMessage::Subscriptions(Response::SubscriptionAdded(id)), sender);
			},
			Msg::SetActors(actors) => {
				self.actors = actors;
			},
		}
	}
}
impl Receive<Request> for SubscriptionsActor {
	type Msg = SubscriptionsActorMsg;
	fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Request, sender: Sender) {
		trace!("Received subscriptions::Request");
		let connection = self.connection.clone();
		let actors = self.actors.clone();
		let myself = ctx.myself();
		ctx.run(async move {
			match msg {
				Request::GetPriceSubscriptionList => {
					debug!("Getting subscription list");
					let list = get_subscription_list().await;
					connection.tell(ServerMessage::Subscriptions(Response::SubscriptionList(list)), sender);
				},
				Request::AddPriceSubscription(request) => {
					debug!("Subscribing to market pair {}", &request.market_pair);
					let id = add_subscription(request.clone()).await.unwrap();
					myself.tell(Msg::AddActor(id.clone()), sender);
				},
				Request::Subscription(id, req) => {
					let id = id.clone();
					trace!("Request for Subscription {:#?}", id);
					if let Some(actor) = actors.get(&id) {
						actor.tell(crate::subscriptions::cache::actor::Msg::from(req), sender);
					} else {
						connection.tell(ServerMessage::Subscriptions(Response::SubscriptionNotFound(id)), sender);
					}
				}
			}
		}).expect("Failed to run future!");
	}
}
