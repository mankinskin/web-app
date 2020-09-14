#![feature(async_closure)]

extern crate serde;
extern crate serde_json;
extern crate openlimits;
extern crate tokio;
extern crate async_std;
extern crate async_h1;
extern crate futures;
extern crate futures_core;
extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate chrono;
extern crate telegram_bot;
extern crate warp;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_appender;

mod server;
mod shared;
use server::{
    error::Error,
    telegram::{
        self,
        Telegram,
    },
    binance::{
        self,
        Binance,
    },
    model::{
        self,
        Model,
    },
    message_stream::{
        MessageStream,
    },
};
use async_std::{
    sync::{
        Arc,
        MutexGuard,
        RwLock,
    },
    stream::{
        Interval,
        interval,
    },
};
use std::{
    time::Duration,
};
use clap::{
    App,
    Arg,
};
use tracing::{
    debug,
};
use tracing_subscriber::{
    fmt,
    layer::{
        SubscriberExt,
    },
};
use tracing_appender::{
    non_blocking::{
        WorkerGuard,
    },
};

pub async fn run_command(text: String) -> Result<String, Error> {
    debug!("Running command...");
    let mut args = vec![""];
    args.extend(text.split(" "));
    let app = App::new("")
        .subcommand(
            App::new("price")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market Symbol to get the price of")
                )
        )
        .subcommand(
            App::new("history")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market symbol to get the history of")
                )
        )
        .subcommand(
            App::new("watch")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market symbol to watch")
                )
        )
        .get_matches_from_safe(args);
    Ok(match app {
        Ok(app) =>
            if let Some(price_app) = app.subcommand_matches("price") {
                if let Some(symbol) = price_app.value_of("symbol") {
                    let price = binance().await.get_symbol_price(symbol).await?;
                    format!("{:#?}", price)
                } else {
                    price_app.usage().to_string() 
                }
            } else if let Some(history_app) = app.subcommand_matches("history") {
                if let Some(symbol) = history_app.value_of("symbol") {
                let price_history = binance().await.get_symbol_price_history(
                        shared::PriceHistoryRequest {
                            market_pair: symbol.to_string().clone(),
                            interval: None,
                            paginator: None,
                        }
                    ).await?;
                    format!("{:#?}", price_history)
                } else {
                    history_app.usage().to_string() 
                }
            } else if let Some(watch_app) = app.subcommand_matches("watch") {
                if let Some(symbol) = watch_app.value_of("symbol") {
                    model().await.add_symbol(symbol.to_string()).await?;
                    INTERVAL.try_write().unwrap()
                        .get_or_insert_with(|| interval(Duration::from_secs(1)));    
                    String::new()
                } else {
                    watch_app.usage().to_string() 
                }
            } else {
                app.usage().to_string() 
            },
        Err(err) => format!("{}", err),
    })
}
pub async fn telegram() -> Telegram {
    telegram::TELEGRAM.clone()
}
pub async fn binance() -> MutexGuard<'static, Binance> {
    binance::BINANCE.lock().await
}
pub async fn model() -> MutexGuard<'static, Model> {
    model::MODEL.lock().await
}
use lazy_static::lazy_static;
lazy_static! {
    static ref INTERVAL: Arc<RwLock<Option<Interval>>> = Arc::new(RwLock::new(None));
}

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter("server=debug")
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer))
    ).expect("Unable to set global tracing subscriber");
    debug!("Tracing initialized.");
    guard
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = init_tracing();
    binance().await.init().await;
    server::run().await?;
    MessageStream::init()
        .await?
        .handle_messages()
        .await
}
