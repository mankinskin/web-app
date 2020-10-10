use crate::server::{
    command::run_command,
    keys,
};
use async_std::sync::{
    channel,
    Arc,
    Receiver,
    RwLock,
};
use futures::StreamExt;
use futures_core::stream::Stream;
use lazy_static::lazy_static;
use std::{
    pin::Pin,
    task::Poll,
};
pub use telegram_bot::{
    Api,
    Error,
    Update,
};
use telegram_bot::{
    CanReplySendMessage,
    Message,
    MessageKind,
    UpdateKind,
};
use tracing::debug;

#[derive(Clone)]
pub struct Telegram {
    pub api: Api,
}
lazy_static! {
    pub static ref TELEGRAM: Telegram = Telegram::new();
    pub static ref STREAM: Arc<RwLock<Option<Receiver<Result<Update, Error>>>>> =
        Arc::new(RwLock::new(None));
}
pub async fn run() {
    let (tx, rx) = channel(100);
    *STREAM.try_write().unwrap() = Some(rx);
    let mut stream = telegram().api.stream();
    while let Some(msg) = stream.next().await {
        tx.send(msg).await
    }
}
pub fn telegram() -> Telegram {
    TELEGRAM.clone()
}
fn remove_coloring(text: String) -> String {
    let reg = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    reg.replace_all(&text, "").to_string()
}
impl Telegram {
    pub fn new() -> Self {
        let telegram_key = keys::read_key_file("keys/telegram");
        let api = Api::new(telegram_key);
        Self { api }
    }
    pub async fn handle_message(&mut self, message: Message) -> Result<(), crate::Error> {
        match message.kind.clone() {
            MessageKind::Text { data, .. } => {
                let cmd = data;
                println!("<{}>: {}", &message.from.first_name, cmd);
                let output = run_command(cmd).await?;
                let result = self
                    .api
                    .send(message.text_reply(format!("{}", remove_coloring(output),)))
                    .await;
                if let Err(e) = result {
                    self.api
                        .send(message.text_reply(format!("{:#?}", e,)))
                        .await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn update(&mut self, update: Update) -> Result<(), crate::Error> {
        debug!("Telegram Update");
        Ok(match update.kind {
            UpdateKind::Message(message) => self.handle_message(message).await?,
            UpdateKind::EditedMessage(_message) => {}
            UpdateKind::ChannelPost(_post) => {}
            UpdateKind::EditedChannelPost(_post) => {}
            UpdateKind::InlineQuery(_query) => {}
            UpdateKind::CallbackQuery(_query) => {}
            UpdateKind::Error(_error) => {}
            UpdateKind::Unknown => {}
        })
    }
}
impl std::ops::Deref for Telegram {
    type Target = Api;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
pub struct TelegramStream;
impl Stream for TelegramStream {
    type Item = Result<Update, crate::Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        //debug!("Polling TelegramStream...");
        if let Some(mut stream_opt) = STREAM.try_write() {
            let poll = if let Some(stream) = &mut *stream_opt {
                let mut stream = stream;
                let telegram_poll = Stream::poll_next(Pin::new(&mut stream), cx);
                if telegram_poll.is_ready() {
                    telegram_poll.map(|opt| {
                        opt.map(|result| {
                            result.map_err(Into::into)
                        })
                    })
                } else {
                    Poll::Pending
                }
            } else {
                Poll::Pending
            };
            if let Poll::Ready(Some(Err(_))) = poll {
                *stream_opt = None;
            }
            poll
        } else {
            Poll::Pending
        }
    }
}
