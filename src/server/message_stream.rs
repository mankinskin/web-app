use crate::{
    Error,
    telegram,
    server::{
        websocket,
        interval,
        telegram::{
            Update,
            TelegramStream,
        },
        command::{
            run_command,
            CommandLineStream,
        },
    },
    shared::{
        ServerMessage,
    },
};
use futures::{
    StreamExt,
};
use futures_core::{
    stream::{
        Stream,
    },
};
use std::{
    pin::Pin,
    task::Poll,
};
use tracing::{
    debug,
    error,
};

#[derive(Debug)]
pub enum Message {
    Telegram(Update),
    CommandLine(String),
    WebSocket(usize, ServerMessage),
    Interval,
}
pub struct MessageStream {
    pub command_line: CommandLineStream,
    pub telegram_stream: TelegramStream,
}
impl MessageStream {
    pub fn init() -> Self {
        debug!("Initializing MessageStream...");
        MessageStream {
            command_line: CommandLineStream::new(),
            telegram_stream: TelegramStream::new(),
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(result) = self.next().await {
            match result {
                Ok(message) => {
                    tokio::spawn(async {
                        if let Err(e) = Self::handle_message(message).await {
                            error!("{:#?}", e);
                        }
                    });
                },
                Err(err) => self.handle_error(err).await?,
            }
        }
        Ok(())
    }
    async fn handle_message(msg: Message) -> Result<(), Error> {
        match msg {
            Message::Telegram(update) => {
                telegram().update(update).await?
            },
            Message::CommandLine(text) => {
                match run_command(text).await {
                    Ok(ok) => println!("{}", ok),
                    Err(err) => error!("{:#?}", err),
                };
            },
            Message::Interval => {
                crate::model().await.update().await?;
                for (_, session) in websocket::sessions().read().await.iter() {
                    session.clone().write().await.send_update().await?;
                }
            },
            Message::WebSocket(id, msg) => {
                websocket::handle_message(id, msg).await?;
            }
        }
        Ok(())
    }
    async fn handle_error(&mut self, error: Error) -> Result<(), Error> {
        error!("{:#?}", error);
        Ok(())
    }
}
impl Stream for MessageStream {
    type Item = Result<Message, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        //debug!("Polling MessageStream...");
        let rself = self.get_mut();
        let interval_poll = Stream::poll_next(Pin::new(&mut interval::IntervalStream), cx);
        if interval_poll.is_ready() {
            return interval_poll;
        }
        let websocket_poll = Stream::poll_next(Pin::new(&mut websocket::SessionsStream), cx);
        if websocket_poll.is_ready() {
            return websocket_poll;
        }
        let cli_poll = Stream::poll_next(Pin::new(&mut rself.command_line), cx);
        if cli_poll.is_ready() {
            return cli_poll;
        }
        let telegram_poll = Stream::poll_next(Pin::new(&mut rself.telegram_stream), cx);
        if telegram_poll.is_ready() {
            return websocket_poll;
        }
        Poll::Pending
    }
}
pub async fn run() -> Result<(), Error> {
    MessageStream::init().run().await
}
