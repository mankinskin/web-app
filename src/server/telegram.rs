use crate::server::{
    command::run_command,
    keys,
};
use futures::StreamExt;
use lazy_static::lazy_static;
pub use telegram_bot::{
    Api,
};
use telegram_bot::{
    CanReplySendMessage,
    MessageKind,
    UpdateKind,
};
use tracing::{
    debug,
    error,
    info,
};
use actix::{
    Actor,
    Handler,
    Context,
    Addr,
    ResponseActFuture,
    StreamHandler,
    AsyncContext,
    Message,
    Supervisor,
};
use actix_interop::{
    FutureInterop,
};

#[derive(Clone, Debug)]
pub struct Error(String);
impl<T: ToString> From<T> for Error {
    fn from(err: T) -> Self {
        Self(err.to_string())
    }
}
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct Update(pub telegram_bot::Update);

#[derive(Clone)]
pub struct StaticTelegram {
    pub api: Api,
}
lazy_static! {
    pub static ref TELEGRAM: StaticTelegram = StaticTelegram::new();
}
pub fn telegram() -> StaticTelegram {
    TELEGRAM.clone()
}
fn remove_coloring(text: String) -> String {
    let reg = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    reg.replace_all(&text, "").to_string()
}
impl StaticTelegram {
    pub fn new() -> Self {
        let telegram_key = keys::read_key_file("keys/telegram");
        let api = Api::new(telegram_key);
        Self { api }
    }
    pub async fn handle_message(&mut self, message: telegram_bot::Message) -> Result<(), Error> {
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
        let Update(update) = update;
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
impl std::ops::Deref for StaticTelegram {
    type Target = Api;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
pub struct Telegram;
impl Telegram {
    pub async fn init() -> Addr<Self> {
        Supervisor::start(|_| Self)
    }
}
impl Actor for Telegram {
    type Context = Context<Self>;
   fn started(&mut self, ctx: &mut Context<Self>) {
       // add stream
       Self::add_stream(telegram().api.stream().map(|res| res.map_err(Into::into)), ctx);
   }
}
impl StreamHandler<Result<telegram_bot::Update, Error>> for Telegram {
    fn handle(
        &mut self,
        msg: Result<telegram_bot::Update, Error>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(msg) => ctx.notify(Update(msg)),
            Err(err) => error!("{:#?}", err),
        }
    }
}
impl Handler<Update> for Telegram {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Update,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        async move {
            if let Err(e) = telegram().update(msg).await {
                error!("{:#?}", e);
            }
        }.interop_actor_boxed(self)
    }
}
impl actix::Supervised for Telegram {
    fn restarting(&mut self, _ctx: &mut Context<Self>) {
        info!["Restarting telegram actor"];
    }
}
