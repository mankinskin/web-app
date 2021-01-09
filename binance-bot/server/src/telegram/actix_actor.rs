use futures::StreamExt;
use actix::{
	Actor,
	Handler,
	Context,
	Addr,
	ResponseActFuture,
	StreamHandler,
	AsyncContext,
	Message,
	Supervisor,
};
use actix_interop::{
	FutureInterop,
};

pub struct Telegram;
impl Telegram {
	pub async fn init() -> Addr<Self> {
		Supervisor::start(|_| Self)
	}
}
impl Actor for Telegram {
	type Context = Context<Self>;
   fn started(&mut self, ctx: &mut Context<Self>) {
	   // add stream
	   Self::add_stream(telegram().api.stream().map(|res| res.map_err(Into::into)), ctx);
   }
}
impl StreamHandler<Result<telegram_bot::Update, Error>> for Telegram {
	fn handle(
		&mut self,
		msg: Result<telegram_bot::Update, Error>,
		ctx: &mut Self::Context,
	) {
		match msg {
			Ok(msg) => ctx.notify(Update(msg)),
			Err(err) => error!("{:#?}", err),
		}
	}
}
impl Handler<Update> for Telegram {
	type Result = ResponseActFuture<Self, ()>;
	fn handle(
		&mut self,
		msg: Update,
		_ctx: &mut Self::Context,
	) -> Self::Result {
		async move {
			if let Err(e) = telegram().update(msg).await {
				error!("{:#?}", e);
			}
		}.interop_actor_boxed(self)
	}
}
impl actix::Supervised for Telegram {
	fn restarting(&mut self, _ctx: &mut Context<Self>) {
		info!["Restarting telegram actor"];
	}
}