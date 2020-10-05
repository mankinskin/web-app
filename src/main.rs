#![feature(async_closure)]
#![feature(bool_to_option)]

extern crate app_model;
extern crate async_h1;
extern crate async_std;
extern crate chrono;
extern crate clap;
extern crate futures;
extern crate futures_core;
extern crate lazy_static;
extern crate openlimits;
extern crate parallel_stream;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate telegram_bot;
extern crate tokio;
extern crate tracing;
extern crate tracing_appender;
extern crate tracing_subscriber;
extern crate warp;

mod server;
mod shared;
use async_std::sync::MutexGuard;
pub use server::*;
use server::{
    binance::{
        self,
        Binance,
    },
    error::Error,
    message_stream,
    model::{
        self,
        Model,
    },
    telegram::{self,},
};
use tracing::debug;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
};

pub async fn binance() -> MutexGuard<'static, Binance> {
    binance::BINANCE.lock().await
}
pub async fn model() -> MutexGuard<'static, Model> {
    model::MODEL.lock().await
}

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
