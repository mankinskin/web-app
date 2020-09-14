use crate::{
    Error,
    telegram,
    INTERVAL,
    server::{
        websocket::{
            self,
            WEBSOCKETS,
        },
        telegram::{
            Update,
        },
        command::{
            run_command,
        },
    },
};
use telegram_bot::{
    UpdatesStream,
};
use futures::{
    StreamExt,
};
use futures_core::{
    stream::{
        Stream,
    },
};
use async_std::{
    io::{
        BufReader,
        prelude::{
            BufReadExt,
        },
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
    WebSocket(usize, warp::ws::Message),
    Model,
}
pub struct MessageStream {
    pub stdin: async_std::io::Stdin,
    pub telegram_stream: Option<UpdatesStream>,
}
impl MessageStream {
    pub async fn init() -> Self {
        debug!("Initializing MessageStream...");
        MessageStream {
            stdin: async_std::io::stdin(),
            telegram_stream: Some(telegram().await.stream()),
        }
    }
    pub async fn handle_messages(&mut self) {
        while let Some(result) = self.next().await {
            match result {
                Ok(message) => {
                    tokio::spawn(async {
                        Self::handle_message(message).await
                    });
                },
                Err(err) => self.handle_error(err).await.unwrap(),
            }
        }
    }
    async fn handle_message(msg: Message) -> Result<(), Error> {
        match msg {
            Message::Telegram(update) => telegram().await.update(update).await.map_err(Into::into),
            Message::CommandLine(text) => Ok(println!("{}", run_command(text).await?)),
            Message::Model => crate::model().await.update().await,
            Message::WebSocket(id, msg) => websocket::handle_message(id, msg).await,
        }
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
        if let Some(mut interval) = INTERVAL.try_write() {
            if let Some(interval) = &mut *interval {
                let interval_poll = Stream::poll_next(Pin::new(interval), cx);
                if interval_poll.is_ready() {
                    //debug!("Interval poll ready");
                    return Poll::Ready(Some(Ok(Message::Model)));
                }
            }
        }
        for (id, (_, receiver)) in WEBSOCKETS.try_write().unwrap().iter_mut() {
            let receiver_poll = Stream::poll_next(Pin::new(receiver), cx);
            if receiver_poll.is_ready() {
                //debug!("WebSocket poll ready");
                return receiver_poll.map(|opt|
                    opt.map(|result|
                        result.map(|msg| Message::WebSocket(id.clone(), msg))
                        .map_err(Into::into)
                    )
                );
            }
        }
        let stdin = BufReader::new(&mut rself.stdin);
        let mut lines = stdin.lines();
        let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
        if cli_poll.is_ready() {
            //debug!("CLI poll ready");
            return cli_poll.map(|opt|
                opt.map(|result|
                    result.map(|line| Message::CommandLine(line))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        if let Some(telegram) = &mut rself.telegram_stream {
            let telegram_poll = Stream::poll_next(Pin::new(telegram), cx);
            if telegram_poll.is_ready() {
                //debug!("Telegram poll ready");
                return telegram_poll.map(|opt|
                    opt.map(|result|
                        match result {
                            Ok(update) => Ok(Message::Telegram(update)),
                            Err(err) => {
                                rself.telegram_stream = None;
                                Err(Error::from(err))
                            },
                        }
                    )
                );
            }
        }
        //debug!("Poll pending.");
        Poll::Pending
    }
}
pub async fn handle_messages() {
    MessageStream::init()
        .await
        .handle_messages()
        .await
}
