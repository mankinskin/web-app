extern crate reqwest;
extern crate telegram_bot;
extern crate serde;
extern crate serde_json;
extern crate binance;
extern crate futures;
extern crate tokio;

use futures::StreamExt;
use telegram_bot::*;

fn read_binance_api_key() -> std::io::Result<String> {
    std::fs::read_to_string("./keys/binance")
        .map(|s| s.trim_end_matches("\n").to_string())
}
fn read_telegram_api_key() -> std::io::Result<String> {
    std::fs::read_to_string("./keys/telegram")
        .map(|s| s.trim_end_matches("\n").to_string())
}

//fn main() {
//    println!("Hello, world!");
//    let addr = "https://api.binance.com";
//    let api_key = read_api_key().expect("Failed to read API key");
//    println!("Using API key: {}", api_key);
//
//}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = read_telegram_api_key().expect("Failed to read keys/telegram");
    //println!("Using API key: {}", token);
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                .await?;
            }
        }
    }
    Ok(())
}
