use crate::{
    shared,
};
use async_std::{
    stream::{
        interval,
    },
};
use std::{
    time::Duration,
};
use clap::{
    App,
    Arg,
};
use tracing::{
    debug,
};

pub async fn run_command(text: String) -> Result<String, crate::Error> {
    debug!("Running command...");
    let mut args = vec![""];
    args.extend(text.split(" "));
    let app = App::new("")
        .subcommand(
            App::new("price")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market Symbol to get the price of")
                )
        )
        .subcommand(
            App::new("history")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market symbol to get the history of")
                )
        )
        .subcommand(
            App::new("watch")
                .arg(
                    Arg::with_name("symbol")
                        .help("The Market symbol to watch")
                )
        )
        .get_matches_from_safe(args);
    Ok(match app {
        Ok(app) =>
            if let Some(price_app) = app.subcommand_matches("price") {
                if let Some(symbol) = price_app.value_of("symbol") {
                    let price = crate::binance().await.get_symbol_price(symbol).await?;
                    format!("{:#?}", price)
                } else {
                    price_app.usage().to_string() 
                }
            } else if let Some(history_app) = app.subcommand_matches("history") {
                if let Some(symbol) = history_app.value_of("symbol") {
                let price_history = crate::binance().await.get_symbol_price_history(
                        shared::PriceHistoryRequest {
                            market_pair: symbol.to_string().clone(),
                            interval: None,
                            paginator: None,
                        }
                    ).await?;
                    format!("{:#?}", price_history)
                } else {
                    history_app.usage().to_string() 
                }
            } else if let Some(watch_app) = app.subcommand_matches("watch") {
                if let Some(symbol) = watch_app.value_of("symbol") {
                    crate::model().await.add_symbol(symbol.to_string()).await?;
                    crate::INTERVAL.try_write().unwrap()
                        .get_or_insert_with(|| interval(Duration::from_secs(1)));    
                    String::new()
                } else {
                    watch_app.usage().to_string() 
                }
            } else {
                app.usage().to_string() 
            },
        Err(err) => format!("{}", err),
    })
}
