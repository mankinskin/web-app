use crate::{
    Error,
    telegram,
    INTERVAL,
    server::{
        websocket::{
            self,
            WebSocketSession,
        },
        telegram::{
            Update,
        },
        command::{
            run_command,
        },
    },
    shared::{
        ClientMessage,
        ServerMessage,
    },
};
use telegram_bot::{
    UpdatesStream,
};
use futures::{
    StreamExt,
    SinkExt,
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
    WebSocket(usize, ServerMessage),
    Interval,
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
    pub async fn handle_messages(&mut self) -> Result<(), Error> {
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
                telegram().await.update(update).await?
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
        if let Some(mut interval) = INTERVAL.try_write() {
            if let Some(interval) = &mut *interval {
                let interval_poll = Stream::poll_next(Pin::new(interval), cx);
                if interval_poll.is_ready() {
                    //debug!("Interval poll ready");
                    return Poll::Ready(Some(Ok(Message::Interval)));
                }
            }
        }
        if let Some(mut sessions) = websocket::sessions().try_write() {
            let mut close = None;
            let mut item = None;
            for (id, session) in sessions.iter_mut() {
                if let Some(mut session) = session.try_write() {
                    let session_poll = Stream::poll_next(Pin::new(&mut *session), cx);
                    if let Poll::Ready(opt) = session_poll {
                        if let Some(result) = opt {
                            item = Some(result
                                    .map(|msg| Message::WebSocket(id.clone(), msg))
                                    .map_err(Error::from));
                        } else {
                            close = Some(id.clone());
                        }
                        break;
                    }
                }
            }
            if let Some(id) = close {
                debug!("Closing WebSocket connection");
                sessions.remove(&id);
            }
            if item.is_some() {
                return Poll::Ready(item);
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
pub async fn handle_messages() -> Result<(), Error> {
    MessageStream::init()
        .await
        .handle_messages()
        .await
}
