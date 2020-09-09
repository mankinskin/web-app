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
    telegram::{
        self,
        Telegram,
        TelegramError,
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

use openlimits::{
    errors::{
        OpenLimitError,
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
    Subscriber,
};
use tracing_subscriber::{
    prelude::*,
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

#[derive(Debug)]
pub enum Error {
    Telegram(TelegramError),
    OpenLimits(OpenLimitError),
    AsyncIO(async_std::io::Error),
    Clap(clap::Error),
    Model(model::Error),
    Tokio(tokio::task::JoinError),
    SerdeJson(serde_json::Error),
    WebSocket(String),
    Warp(warp::Error),
}
impl From<warp::Error> for Error {
    fn from(err: warp::Error) -> Self {
        Self::Warp(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}
impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        Self::Clap(err)
    }
}
impl From<TelegramError> for Error {
    fn from(err: TelegramError) -> Self {
        Self::Telegram(err)
    }
}
impl From<OpenLimitError> for Error {
    fn from(err: OpenLimitError) -> Self {
        Self::OpenLimits(err)
    }
}
impl From<async_std::io::Error> for Error {
    fn from(err: async_std::io::Error) -> Self {
        Self::AsyncIO(err)
    }
}
impl From<model::Error> for Error {
    fn from(err: model::Error) -> Self {
        Self::Model(err)
    }
}
impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Tokio(err)
    }
}
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
        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter("server")
            .with_max_level(tracing::Level::DEBUG)
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
