extern crate telegram_bot;
extern crate serde;
extern crate serde_json;
extern crate openlimits;
extern crate tokio;
extern crate futures;
extern crate async_std;
extern crate futures_core;
extern crate async_tls;
extern crate rustls;
extern crate async_h1;
extern crate http_types;
extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate chrono;

mod socket;
use socket::{
    TcpSocket,
};
mod telegram;
use telegram::{
    Telegram,
    TelegramError,
    TelegramUpdate,
};
mod shared;
mod binance;
use binance::{
    Binance,
};
mod model;
use model::{
    Model,
};
mod message_stream;
use message_stream::{
    MessageStream,
};

use futures::{
    StreamExt,
};
use openlimits::{
    errors::{
        OpenLimitError,
    },
};
use async_std::{
    net::{
        TcpStream,
    },
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

#[derive(Debug)]
pub enum Error {
    Telegram(TelegramError),
    OpenLimits(OpenLimitError),
    AsyncIO(async_std::io::Error),
    Http(http_types::Error),
    Clap(clap::Error),
    Model(model::Error),
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
impl From<http_types::Error> for Error {
    fn from(err: http_types::Error) -> Self {
        Self::Http(err)
    }
}
impl From<model::Error> for Error {
    fn from(err: model::Error) -> Self {
        Self::Model(err)
    }
}
async fn run_command(text: String) -> Result<String, Error> {
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
                let price_history = crate::binance().await.get_symbol_price_history(
                        binance::PriceHistoryRequest {
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
pub async fn handle_connection(stream: TcpStream) -> Result<(), Error> {
    println!("starting new connection from {}", stream.peer_addr()?);
    let stream = stream.clone();
        if let Err(e) = async_h1::accept(stream, |req| async move {
            TcpSocket::handle_request(req).await
        })
        .await {
            eprintln!("{}", e);
        }
    Ok(())
}
#[derive(Clone, Debug)]
pub enum Update {
    Telegram(TelegramUpdate),
    CommandLine(String),
    TcpStream(TcpStream),
    Interval,
}
async fn handle_update(update: Update) -> Result<(), Error> {
    match update {
        Update::Telegram(update) => telegram().await.update(update).await.map_err(Into::into),
        Update::CommandLine(text) => Ok(println!("{}", run_command(text).await?)),
        Update::TcpStream(stream) => handle_connection(stream).await,
        Update::Interval => crate::model().await.update().await,
    }
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
#[tokio::main]
async fn main() -> Result<(), Error> {
    binance().await.init().await;
    let mut stream = MessageStream::init().await?;
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                tokio::spawn(async {
                    handle_update(update).await.unwrap()
                }).await.unwrap();
            },
            Err(err) => println!("{:#?}", err),
        }
    }
    Ok(())
}
