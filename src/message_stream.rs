use crate::{
    Error,
    telegram,
    INTERVAL,
    telegram::{
        TelegramUpdate,
    },
    run_command,
};
use telegram_bot::{
    UpdatesStream,
};
use futures::{
    StreamExt,
    FutureExt,
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
    net::{
        SocketAddr,
    },
    sync::{
        Arc,
        RwLock,
    },
    stream::{
        Interval,
    },
};
use std::{
    pin::Pin,
    task::Poll,
};
use warp::{
    Filter,
};

const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";
async fn run_server() {
    warp::serve(
        warp::path("ws")
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                // And then our closure will be called when it completes...
                ws.on_upgrade(|websocket| {
                    // Just echo all messages back...
                    let (tx, rx) = websocket.split();
                    rx.forward(tx).map(|result| {
                        if let Err(e) = result {
                            eprintln!("websocket error: {:?}", e);
                        }
                    })
                })
            })
            .or(warp::fs::dir(PKG_PATH.to_string()))

        )
        .run(SocketAddr::from(([0,0,0,0], 8000)))
        .await
}
#[derive(Debug)]
pub enum Message {
    Telegram(TelegramUpdate),
    CommandLine(String),
    Model,
}
pub struct MessageStream {
    pub stdin: async_std::io::Stdin,
    pub telegram_stream: Option<UpdatesStream>,
    pub interval: Arc<RwLock<Option<Interval>>>,
}
impl MessageStream {
    pub async fn init() -> Result<Self, Error> {
        tokio::spawn(run_server());
        Ok(MessageStream {
            stdin: async_std::io::stdin(),
            telegram_stream: Some(telegram().await.stream()),
            interval: INTERVAL.clone(),
        })
    }
    async fn handle_message(msg: Message) -> Result<(), Error> {
        match msg {
            Message::Telegram(update) => telegram().await.update(update).await.map_err(Into::into),
            Message::CommandLine(text) => Ok(println!("{}", run_command(text).await?)),
            Message::Model => crate::model().await.update().await,
        }
    }
    async fn handle_error(&mut self, error: Error) -> Result<(), Error> {
        Ok(println!("{:#?}", error))
    }
    pub async fn handle_messages(&mut self) -> Result<(), Error> {
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
        Ok(())
    }
}
impl Stream for MessageStream {
    type Item = Result<Message, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let rself = self.get_mut();
        if let Some(mut interval) = rself.interval.try_write() {
            if let Some(interval) = &mut *interval {
                let interval_poll = Stream::poll_next(Pin::new(interval), cx);
                if interval_poll.is_ready() {
                    return Poll::Ready(Some(Ok(Message::Model)));
                }
            }
        }
        let stdin = BufReader::new(&mut rself.stdin);
        let mut lines = stdin.lines();
        let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
        if cli_poll.is_ready() {
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
        Poll::Pending
    }
}
