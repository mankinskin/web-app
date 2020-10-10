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
    websocket,
    interval,
    error::Error,
    message_stream,
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
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = init_tracing();
    binance().await.init().await;
    let msg_stream = message_stream::MessageStream::new()
        .with_stream(interval::IntervalStream)
        .with_stream(websocket::Connections)
        .with_stream(command::CommandLine)
        .with_stream(telegram::TelegramStream);
    let msg_stream_future = msg_stream.spawn_handlers(|msg: Message| async {
            match msg {
                Message::Telegram(update) => telegram().update(update).await?,
                Message::CommandLine(text) => {
                    match run_command(text).await {
                        Ok(ok) => println!("{}", ok),
                        Err(err) => error!("{:#?}", err),
                    };
                }
                Message::Interval => {
                    crate::subscriptions().await.update().await?;
                    websocket::update().await?;
                }
                Message::WebSocket(id, msg) => {
                    debug!("Websocket message from connection {} {:?}", id, msg);
                    if let Err(err) = websocket::handle_message(id, msg).await {
                        error!("{:#?}", err);
                    }
                }
            }
            Ok(())
        });
    let (_telegram_result, _server_result, ms_result) = futures::join! {
        telegram::run(),
        server::listen(),
        msg_stream_future,
    };
    ms_result
}
