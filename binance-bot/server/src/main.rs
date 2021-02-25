#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]
pub mod binance;
pub mod command;
pub mod error;
pub mod keys;
pub mod subscriptions;
pub mod telegram;
pub mod database;
pub mod websocket;
pub mod server;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
	fmt::{
		Layer,
		Subscriber,
	},
	layer::SubscriberExt,
};
#[allow(unused)]
use tracing::{
	debug,
	info,
	error,
	warn,
	trace,
};
use async_std::{
	sync::{
		Arc,
		RwLock,
		RwLockReadGuard,
		RwLockWriteGuard,
	},
};
use riker::actors::*;
use lazy_static::lazy_static;
use std::fmt::{
	Formatter,
	Display,
	self,
};
use const_format::formatcp;
pub use server::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let _tracing = init_tracing();
	server::run().await
}

pub const CLIENT_PATH: &str = "../client";
pub const PKG_PATH: &str = formatcp!("{}/pkg", CLIENT_PATH);

lazy_static! {
	static ref ACTOR_SYS: Arc<RwLock<ActorSystem>> = Arc::new(RwLock::new(ActorSystem::new().unwrap()));
}
pub async fn actor_sys() -> RwLockReadGuard<'static, ActorSystem> {
	ACTOR_SYS.read().await
}
pub async fn actor_sys_mut() -> RwLockWriteGuard<'static, ActorSystem> {
	ACTOR_SYS.write().await
}
pub fn try_actor_sys() -> Option<RwLockReadGuard<'static, ActorSystem>> {
    ACTOR_SYS.try_read()
}
pub fn try_actor_sys_mut() -> Option<RwLockWriteGuard<'static, ActorSystem>> {
    ACTOR_SYS.try_write()
}
#[derive(Debug, Clone)]
pub struct Error(String);
impl From<String> for Error {
	fn from(s: String) -> Self {
		Self(s)
	}
}
impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let Self(s) = self;
		write!(f, "{}", s)
	}
}
pub fn init_tracing() -> WorkerGuard {
	tracing_log::LogTracer::init().unwrap();
	let file_appender = tracing_appender::rolling::hourly("./logs", "log");
	let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
	let subscriber = Subscriber::builder()
			.with_env_filter("hyper=error,reqwest=error,h2=error,[]=debug")
			.finish()
			.with(Layer::default().with_writer(file_writer));
	tracing::subscriber::set_global_default(subscriber)
		.expect("Unable to set global tracing subscriber");
	info!("Tracing initialized.");
	info!["Info logs enabled"];
	trace!["Trace logs enabled"];
	debug!["Debug logs enabled"];
	warn!["Warning logs enabled"];
	error!["Error logs enabled"];
	guard
}
