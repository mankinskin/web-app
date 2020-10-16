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
pub use subscriptions::subscriptions;
pub use binance::binance;
use crate::shared::ClientMessage;
use crate::{
    telegram::{
        self,
        Update,
    },
    websocket::{
        self,
    },
    interval,
    error::Error,
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
    binance().await.init().await;
    server::run().await
}
