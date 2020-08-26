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
use telegram_bot::{
    UpdatesStream,
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

use futures_core::{
    stream::{
        Stream,
    },
};
use futures::{
    StreamExt,
};
use openlimits::{
    errors::{
        OpenLimitError as BinanceError,
    },
};
use async_std::{
    io::{
        BufReader,
        prelude::{
            BufReadExt,
        },
    },
    net::{
        Incoming,
        TcpStream,
        TcpListener,
        SocketAddr,
    },
    sync::{
        Arc,
    },
};
use std::{
    pin::Pin,
    task::Poll,
};
use rustls::{
    ServerConfig,
    NoClientAuth,
};
use async_tls::{
    TlsAcceptor,
};

#[derive(Debug)]
pub enum Error {
    Telegram(TelegramError),
    Binance(BinanceError),
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
impl From<BinanceError> for Error {
    fn from(err: BinanceError) -> Self {
        Self::Binance(err)
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
struct MessageStream<'a> {
    pub telegram_stream: UpdatesStream,
    pub stdin: async_std::io::Stdin,
    pub incoming: Incoming<'a>,
}
impl<'a> Stream for MessageStream<'a> {
    type Item = Result<Update, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let rself = self.get_mut();
        let stdin = BufReader::new(&mut rself.stdin);
        let mut lines = stdin.lines();
        let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
        if cli_poll.is_ready() {
            return cli_poll.map(|opt|
                opt.map(|result|
                    result.map(|line| Update::CommandLine(line))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        let telegram_poll = Stream::poll_next(Pin::new(&mut rself.telegram_stream), cx);
        if telegram_poll.is_ready() {
            return telegram_poll.map(|opt|
                opt.map(|result|
                    result.map(|update| Update::Telegram(update))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        let incoming_poll = Stream::poll_next(Pin::new(&mut rself.incoming), cx);
        if incoming_poll.is_ready() {
            return incoming_poll.map(|opt|
                opt.map(|result|
                    result.map(|stream| Update::TcpStream(stream))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        Poll::Pending
    }
}
use clap::{
    App,
    Arg,
};
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
        .get_matches_from_safe(args);
    Ok(match app {
        Ok(app) =>
            if let Some(price_app) = app.subcommand_matches("price") {
                if let Some(symbol) = price_app.value_of("symbol") {
                    let price = binance().await.get_symbol_price(symbol).await;
                    format!("{:#?}", price)
                } else {
                    price_app.usage().to_string() 
                }
            } else if let Some(history_app) = app.subcommand_matches("history") {
                if let Some(symbol) = history_app.value_of("symbol") {
                    let price_history = binance().await.get_symbol_price_history(symbol).await;
                    format!("{:#?}", price_history)
                } else {
                    history_app.usage().to_string() 
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
enum Update {
    Telegram(TelegramUpdate),
    CommandLine(String),
    TcpStream(TcpStream),
}
async fn handle_update(update: Update) -> Result<(), Error> {
    match update {
        Update::Telegram(update) => telegram().await.update(update).await.map_err(Into::into),
        Update::CommandLine(text) => Ok(println!("{}", run_command(text).await?)),
        Update::TcpStream(stream) => handle_connection(stream).await,
    }
}
pub async fn telegram() -> Telegram {
    telegram::TELEGRAM.clone()
}
pub async fn binance() -> Binance {
    binance::BINANCE.clone()
}
pub async fn model() -> MutexGuard<'static, Model> {
    model::MODEL.lock().await
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = ServerConfig::new(NoClientAuth::new());
    let mut _acceptor = TlsAcceptor::from(Arc::new(config));
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let listener = TcpListener::bind(addr).await?;

    let mut stream = MessageStream {
        telegram_stream: telegram().await.stream(),
        stdin: async_std::io::stdin(),
        incoming: listener.incoming(),
    };

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
