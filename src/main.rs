#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]

mod server;
mod shared;
pub use server::*;
use tracing::{
    debug,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
};
use crate::{
    websocket::{
        self,
    },
    error::Error,
};

fn init_tracing() -> WorkerGuard {
    tracing_log::LogTracer::init_with_filter(log::LevelFilter::Trace).unwrap();
    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer)),
    )
    .expect("Unable to set global tracing subscriber");
    debug!("Tracing initialized.");
    guard
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _guard = init_tracing();
    server::run().await
}
