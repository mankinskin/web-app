use crate::shared;
use telegram_bot::{
    *,
    UpdateKind,
};
pub use telegram_bot::{
    Error as TelegramError,
    Update as TelegramUpdate,
};
use lazy_static::lazy_static;

#[derive(Clone)]
pub struct Telegram {
    api: Api,
}
lazy_static! {
    pub static ref TELEGRAM: Telegram = Telegram::new();
}
impl Telegram {
    pub fn new() -> Self {
        let telegram_key = shared::read_key_file("keys/telegram");
        let api = Api::new(telegram_key);
        Self {
            api,
        }
    }
    pub async fn handle_message(&mut self, message: Message) -> Result<(), Error> {
        match message.kind.clone() {
            MessageKind::Text { data, .. } => {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);
                let btc_price = crate::binance().await.get_symbol_price(data).await;
        
                self.api.send(message.text_reply(format!(
                    "{:#?}", btc_price,
                )))
                .await?;
            },
            _ => {},
        }
        Ok(())
    }
    pub async fn update(&mut self, update: TelegramUpdate) -> Result<(), Error> {
        Ok(
            match update.kind {
                UpdateKind::Message(message) => {
                    self.handle_message(message).await?
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
impl std::ops::Deref for Telegram {
    type Target = Api;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
