use crate::{
	command::run_command,
	keys,
};
use lazy_static::lazy_static;
pub use telegram_bot::{
	Api,
};
use telegram_bot::{
	CanReplySendMessage,
	MessageKind,
	UpdateKind,
	Update,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
};
pub mod actor;

pub use actor::TelegramActor;

#[derive(Clone, Debug)]
pub struct Error(String);
impl<T: ToString> From<T> for Error {
	fn from(err: T) -> Self {
		Self(err.to_string())
	}
}
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
		let telegram_key = keys::read_key_file("telegram");
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
	pub async fn update(&mut self, update: Update) -> Result<(), Error> {
		debug!("Telegram Update");
		Ok(match update.kind {
			UpdateKind::Message(message) => self.handle_message(message).await?,
			UpdateKind::EditedMessage(_message) => {}
			UpdateKind::ChannelPost(_post) => {}
			UpdateKind::EditedChannelPost(_post) => {}
			UpdateKind::InlineQuery(_query) => {}
			UpdateKind::CallbackQuery(_query) => {}
			UpdateKind::Error(_error) => {}
			_ => {}
		})
	}
}
impl std::ops::Deref for StaticTelegram {
	type Target = Api;
	fn deref(&self) -> &Self::Target {
		&self.api
	}
}
