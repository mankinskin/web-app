#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]

mod server;
mod shared;
use async_std::sync::MutexGuard;
pub use server::*;
use server::{
    error::Error,
    message_stream,
    telegram::{self,},
};
use tracing::debug;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
};
pub use subscriptions::subscriptions;
pub use binance::binance;

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter("server=debug")
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer)),
    )
    .expect("Unable to set global tracing subscriber");
    debug!("Tracing initialized.");
    guard
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = init_tracing();
    binance().await.init().await;
    let (_telegram_result, _server_result, ms_result) = futures::join! {
        telegram::run(),
        server::listen(),
        message_stream::run(),
    };
    ms_result
}
