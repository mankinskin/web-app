#![feature(async_closure)]

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
    error,
    info,
    trace,
    warn,
};

mod server;

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let _tracing = init_tracing();
	server::run().await
}
