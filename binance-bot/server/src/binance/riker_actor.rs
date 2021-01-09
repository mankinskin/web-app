
use riker::actors::*;
use crate::binance::{
	self,
	binance,
};

#[derive(Debug)]
pub struct BinanceActor;

impl ActorFactory for BinanceActor {
	fn create() -> Self {
		Self
	}
}
impl Actor for BinanceActor {
	type Msg = ();
	fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
		ctx.run(async move {
			if !binance().await.is_initialized().await {
				binance::init_api().await;
			}
		}).expect("Failed to run binance initialization!");
	}
	fn recv(&mut self, _ctx: &Context<Self::Msg>, _msg: Self::Msg, _sender: Option<BasicActorRef>) {
	}
}
