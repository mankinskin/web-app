extern crate reqwest;
extern crate telegram_bot;
extern crate serde;
extern crate serde_json;
extern crate binance;
extern crate tokio;
extern crate futures;
extern crate async_std;
extern crate futures_core;

use futures_core::{
    stream::{
        Stream,
    },
};
use futures::{
    StreamExt,
};
use telegram_bot::{
    *,
    Error as TelegramError,
    Update as TelegramUpdate,
};
use binance::{
    model::{
        SymbolPrice,
    },
    market::{
        Market,
    },
    api::{
        Binance,
    },
    errors::{
        Error as BinanceError,
    },
};
use std::{
    convert::{
        AsRef,
    },
    path::{
        Path,
    },
};
use async_std::{
    io::{
        BufReader,
        prelude::BufReadExt,
    },
};
fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
fn setup_binance_market() -> Market {
    let binance_api_key = read_key_file("keys/binance_api");
    let binance_secret_key = read_key_file("keys/binance_secret");

    Market::new(Some(binance_api_key), Some(binance_secret_key))
}
fn setup_telegram_api() -> Api {
    let telegram_key = read_key_file("keys/telegram");
    Api::new(telegram_key)
}
struct Context {
    binance: Market,
    telegram: Api,
}
#[derive(Clone, Debug)]
enum CommandContext {
    Message(Message),
    CommandLine(String),
}
impl From<Message> for CommandContext {
    fn from(m: Message) -> CommandContext {
        Self::Message(m)
    }
}
impl From<String> for CommandContext {
    fn from(m: String) -> CommandContext {
        Self::CommandLine(m)
    }
}
#[derive(Clone, Debug)]
enum Update {
    Telegram(TelegramUpdate),
    CommandLine(String),
}
#[derive(Debug)]
enum Error {
    Telegram(TelegramError),
    Binance(BinanceError),
    CommandLine(async_std::io::Error),
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
        Self::CommandLine(err)
    }
}
struct CommandStream {
    pub telegram_stream: UpdatesStream,
    pub stdin: async_std::io::Stdin,
}
use std::pin::Pin;
use std::task::Poll;
impl Stream for CommandStream {
    type Item = Result<Update, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let rself = self.get_mut();
        let stdin = BufReader::new(&mut rself.stdin);
        let mut lines = stdin.lines();
        let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
        if let Poll::Ready(opt) = cli_poll {
            Poll::Ready(
                opt.map(|result| result.map(|line| Update::CommandLine(line))
                   .map_err(|err| Error::from(err)))
            )
        } else {
            Stream::poll_next(Pin::new(&mut rself.telegram_stream), cx)
                .map(|opt|
                    opt.map(|result|
                        result.map(|update| Update::Telegram(update))
                              .map_err(|err| Error::Telegram(err))
                    )
                )
        }
    }
}
impl Context {
    pub fn new() -> Self {
        Self {
            binance: setup_binance_market(),
            telegram: setup_telegram_api(),
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        let mut stream = CommandStream {
            telegram_stream: self.telegram.stream(),
            stdin: async_std::io::stdin(),
        };
        while let Some(result) = stream.next().await {
            match result {
                Ok(update) => match update {
                    Update::Telegram(update) => self.telegram_update(update).await?,
                    Update::CommandLine(text) => self.run_command(CommandContext::from(text)).await?,
                },
                Err(err) => println!("{:#?}", err),
            }
        }
        Ok(())
    }
    async fn get_price(&mut self, symbol: String) -> Result<SymbolPrice, BinanceError> {
        let binance = self.binance.clone();
        tokio::task::spawn_blocking(move || {
            binance.get_price(symbol.to_uppercase())
        })
        .await.unwrap()
    }
    async fn run_command(&mut self, context: CommandContext) -> Result<(), Error> {
        Ok(match context {
            CommandContext::Message(message) => {
                match message.kind.clone() {
                    MessageKind::Text { data, .. } => {
                        // Print received text message to stdout.
                        println!("<{}>: {}", &message.from.first_name, data);
                        let btc_price = self.get_price(data).await;
                        self.telegram.send(message.text_reply(format!(
                            "{:#?}", btc_price,
                        )))
                        .await?;
                    },
                    _ => {},
                }
            },
            CommandContext::CommandLine(text) => {
                // Print received text message to stdout.
                let btc_price = self.get_price(text).await;
                println!("{:#?}", btc_price);
            },
        })
    }
    async fn telegram_update(&mut self, update: TelegramUpdate) -> Result<(), Error> {
        Ok(
            match update.kind {
                UpdateKind::Message(message) => {
                    self.run_command(CommandContext::from(message)).await?
                },
                UpdateKind::EditedMessage(_message) => {},
                UpdateKind::ChannelPost(_post) => { },
                UpdateKind::EditedChannelPost(_post) => { },
                UpdateKind::InlineQuery(_query) => { },
                UpdateKind::CallbackQuery(_query) => { },
                UpdateKind::Error(_error) => { },
                UpdateKind::Unknown => { },
            }
        )
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    Context::new().run().await
}
