#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]

mod server;
mod shared;
pub use server::*;
use tracing::{
    debug,
    error,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
};
pub use subscriptions::subscriptions;
pub use binance::binance;
use crate::shared::ClientMessage;
use crate::{
    command::{
        run_command,
    },
    telegram::{
        self,
        telegram,
        Update,
    },
    websocket::{
        self,
        ConnectionClientMessage,
    },
    interval,
    error::Error,
    message_stream::EventManager,
};

#[derive(Debug)]
pub enum Message {
    Telegram(Update),
    CommandLine(String),
    WebSocket(usize, ClientMessage),
    Interval,
}
impl From<interval::Msg> for Message {
    fn from(_: interval::Msg) -> Self {
        Message::Interval
    }
}
impl From<command::Msg> for Message {
    fn from(msg: command::Msg) -> Self {
        match msg {
            command::Msg::Line(line) => Message::CommandLine(line),
        }
    }
}
impl From<telegram::Update> for Message {
    fn from(update: telegram::Update) -> Self {
        Message::Telegram(update)
    }
}
impl From<websocket::ConnectionClientMessage> for Message {
    fn from(msg: websocket::ConnectionClientMessage) -> Self {
        Message::WebSocket(msg.0, msg.1)
    }
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
use crate::message_stream::Events;

#[tokio::main]
async fn main() {
    let _guard = init_tracing();
    binance().await.init().await;
    Events::stream(interval::IntervalStream).await;
    Events::stream(websocket::Connections).await;
    Events::stream(telegram::TelegramStream).await;
    Events::stream(command::CommandLine).await;
    //Events::handler(async move |update: telegram_bot::Update| {
    //    if let Err(e) = telegram().update(update).await {
    //        error!("{:#?}", e);
    //    };
    //}).await;
    // TODO add type for CLI command
    Events::handler(async move |msg: command::Msg| {
        debug!("{:#?}", msg)
    }).await;
    Events::handler(async move |_: interval::Msg| {
        //crate::subscriptions().await.update().await.unwrap();
        websocket::update().await.unwrap();
    }).await;
    Events::handler(async move |msg: ConnectionClientMessage| {
        let ConnectionClientMessage(id, msg) = msg;
        if let Err(err) = websocket::handle_message(id, msg).await {
            error!("{:#?}", err);
        }
    }).await;
    futures::join! {
        telegram::run(),
        server::listen(),
        Events::listen(),
    };
}
