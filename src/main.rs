extern crate reqwest;
extern crate telegram_bot;
extern crate serde;
extern crate serde_json;
extern crate binance;
extern crate tokio;
extern crate futures;

use futures::StreamExt;
use telegram_bot::{
    *,
};
use binance::{
    market::{
        Market,
    },
    api::{
        Binance,
    },
};
use std::{
    convert::{
        AsRef,
    },
    path::{
        Path,
    },
};
fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
fn setup_binance_market() -> Market {
    let binance_api_key = read_key_file("keys/binance_api");
    let binance_secret_key = read_key_file("keys/binance_secret");

    Market::new(Some(binance_api_key), Some(binance_secret_key))
}
fn setup_telegram_api() -> Api {
    let telegram_key = read_key_file("keys/telegram");
    Api::new(telegram_key)
}
struct Context {
    binance: Market,
    telegram: Api,
}
impl Context {
    pub fn new() -> Self {
        Self {
            binance: setup_binance_market(),
            telegram: setup_telegram_api(),
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        // Fetch new updates via long poll method
        let mut stream = self.telegram.stream();
        while let Some(update) = stream.next().await {
            // If the received update contains a new message...
            self.update(update?).await?;
        }
        Ok(())
    }
    async fn update(&mut self, update: Update) -> Result<(), Error> {
        Ok(
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { data, .. } = message.clone().kind {
                    // Print received text message to stdout.
                    println!("<{}>: {}", &message.from.first_name, data);
                    let binance = self.binance.clone();
                    let btc_price = tokio::task::spawn_blocking(move || {
                        binance.get_price(data)
                    })
                    .await.unwrap();
                    self.telegram.send(message.text_reply(format!(
                        "BTC price is {:#?}", btc_price,
                    )))
                    .await?;
                }
            }
        )
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut context = Context::new();
    context.run().await
}
