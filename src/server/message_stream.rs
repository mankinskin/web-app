use crate::shared::ClientMessage;
use crate::{
    server::{
        command::{
            run_command,
            CommandLine,
        },
        interval,
        telegram::{
            TelegramStream,
            Update,
        },
        websocket,
    },
    telegram::telegram,
    Error,
};
use futures_core::stream::Stream;
use parallel_stream::ParallelStream;
use std::{
    pin::Pin,
    task::Poll,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
};

#[derive(Debug)]
pub enum Message {
    Telegram(Update),
    CommandLine(String),
    WebSocket(usize, ClientMessage),
    Interval,
}
pub struct MessageStream;
impl MessageStream {
    async fn spawn_handler(message: Message) {
        tokio::spawn(async {
            if let Err(e) = Self::handle_message(message).await {
                error!("{:#?}", e);
            }
        });
    }
    async fn handle_stream_result(result: Result<Message, Error>) {
        match result {
            Ok(message) => Self::spawn_handler(message).await,
            Err(e) => error!("{:#?}", e),
        }
    }
    async fn handle_message(msg: Message) -> Result<(), Error> {
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
    }
}
pub async fn run() -> Result<(), Error> {
    let mut stream = parallel_stream::from_stream(MessageStream);
    while let Some(result) = stream.next().await {
        MessageStream::handle_stream_result(result).await
    }
    Ok(())
}
impl Stream for MessageStream {
    type Item = Result<Message, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        //debug!("Polling MessageStream...");
        let interval_poll = Stream::poll_next(Pin::new(&mut interval::IntervalStream), cx);
        if interval_poll.is_ready() {
            //debug!("Interval poll ready");
            return interval_poll;
        }
        let websocket_poll = Stream::poll_next(Pin::new(&mut websocket::Connections), cx);
        if websocket_poll.is_ready() {
            //debug!("Websocket poll ready: {:?}", websocket_poll);
            return websocket_poll;
        }
        let cli_poll = Stream::poll_next(Pin::new(&mut CommandLine), cx);
        if cli_poll.is_ready() {
            return cli_poll;
        }
        let telegram_poll = Stream::poll_next(Pin::new(&mut TelegramStream), cx);
        if telegram_poll.is_ready() {
            return telegram_poll;
        }
        Poll::Pending
    }
}
