use crate::{
	subscriptions::SubscriptionsActor,
	websocket,
};
use shared::{
	ClientMessage,
	ServerMessage,
	WebsocketCommand,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
};
use riker::actors::*;
use riker::actors::{
	Sender as RkSender,
	CreateError
};
use futures::{
	channel::mpsc::{
		Sender,
	},
};

#[derive(Clone, Debug)]
pub enum Msg {
	SetSubscriptions(ActorRef<<SubscriptionsActor as Actor>::Msg>),
}
#[actor(WebsocketCommand, ServerMessage, ClientMessage, Msg)]
#[derive(Debug)]
pub struct ConnectionActor {
	id: usize,
	sender: Sender<ServerMessage>,
	subscriptions: Option<ActorRef<<SubscriptionsActor as Actor>::Msg>>,
}
impl ConnectionActor {
	pub fn actor_name(id: usize) -> String {
		format!("Connection_{}", id)
	}
	pub async fn create(sender: Sender<ServerMessage>) -> Result<ActorRef<<ConnectionActor as Actor>::Msg>, CreateError> {
		let id = websocket::new_connection_id();
		crate::actor_sys().await.actor_of_args::<ConnectionActor, _>(&Self::actor_name(id), (id, sender))
	}
}
impl Actor for ConnectionActor {
	type Msg = ConnectionActorMsg;
	fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
		debug!("Starting connection actor");
		let myself = ctx.myself();
		let id = self.id.clone();
		ctx.run(async move {
			let actor = SubscriptionsActor::create(id, myself.clone())
				.await
				.expect("Failed to create SubscriptionsActor!");
			myself.tell(Msg::SetSubscriptions(actor), None);
		}).expect("Failed to run future!");
	}
	fn post_stop(&mut self) {
		debug!("Stopped connection actor");
	}
	fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: RkSender) {
		self.receive(ctx, msg, sender);
	}
}
impl Receive<WebsocketCommand> for ConnectionActor {
	type Msg = ConnectionActorMsg;
	fn receive(&mut self, ctx: &Context<Self::Msg>, msg: WebsocketCommand, sender: RkSender) {
		trace!("WebsocketCommand in ConnectionActor");
		//debug!("Received {:#?}", msg);
		match msg {
			WebsocketCommand::Close => ctx.stop(ctx.myself()),
			WebsocketCommand::ClientMessage(msg) => self.receive(ctx, msg, sender),
			_ => {}
		}
	}
}
impl Receive<Msg> for ConnectionActor {
	type Msg = ConnectionActorMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: Msg, _sender: RkSender) {
		match msg {
			Msg::SetSubscriptions(actor) => self.subscriptions = Some(actor),
		}
	}
}
impl Receive<ClientMessage> for ConnectionActor {
	type Msg = ConnectionActorMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ClientMessage, sender: RkSender) {
		trace!("ClientMessage in ConnectionActor");
		match msg {
			ClientMessage::Subscriptions(req) => if let Some(actor) = &self.subscriptions {
				actor.tell(req, sender);
			} else {
				error!("SubscriptionsActor not initialized!");
			},
		}
	}
}
impl Receive<ServerMessage> for ConnectionActor {
	type Msg = ConnectionActorMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ServerMessage, _sender: RkSender) {
		trace!("ServerMessage in ConnectionActor");
		self.sender.try_send(msg).unwrap()
	}
}
impl ActorFactoryArgs<(usize, Sender<ServerMessage>)> for ConnectionActor {
	fn create_args((id, sender): (usize, Sender<ServerMessage>)) -> Self {
		debug!("Creating ConnectionActor");
		Self {
			id,
			sender,
			subscriptions: None,
		}
	}
}
