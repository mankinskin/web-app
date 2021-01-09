
use actix::{
	Actor,
	Context,
	Addr,
};
use actix_web::ResponseError;

impl ResponseError for Error {}
impl Actor for Binance {
	type Context = Context<Self>;
}
pub struct BinanceActor;

impl BinanceActor {
	pub async fn init() -> Addr<Self> {
		init_api().await;
		Self::create(move |_| Binance)
	}
}