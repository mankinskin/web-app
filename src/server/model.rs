use lazy_static::lazy_static;
use openlimits::{
    model::{
        Candle,
    },
};
use async_std::{
    sync::{
        Arc,
        Mutex,
    },
};
use std::{
    collections::{
        HashMap,
    },
};
use chrono::{
    DateTime,
    Utc,
};
use crate::{
    shared,
};
use serde::{
    Serialize,
    Deserialize,
};
use tracing::{
    debug,
};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Error(String);

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s)
    }
}

lazy_static! {
    pub static ref MODEL: Arc<Mutex<Model>> = Arc::new(Mutex::new(Model::new()));
}
#[derive(Default, Serialize, Deserialize)]
pub struct SymbolModel {
    symbol: String,
    prices: Vec<Candle>,
    last_update: Option<DateTime<Utc>>,
}
impl SymbolModel {
    pub fn from_symbol(symbol: String) -> Self {
        Self{
            symbol,
            prices: Vec::new(),
            last_update: None,
        }
    }
    pub async fn update(&mut self) -> Result<(), crate::Error> {
        debug!("SymbolModel update");
        let paginator = self.last_update.and_then(|date_time|
                date_time.timestamp().try_into().ok()
            ).map(|timestamp: u64| 
                        openlimits::model::Paginator {
                            start_time: Some(timestamp),
                            ..Default::default()
                        }
                    );
        let prices = crate::binance().await.get_symbol_price_history(
                shared::PriceHistoryRequest {
                    market_pair: self.symbol.clone(),
                    interval: None,
                    paginator,
                }
            ).await?;
        self.prices = prices;
        self.last_update = Some(Utc::now());
        Ok(())
    }
}
#[derive(Default)]
pub struct Model {
    symbols: HashMap<String, SymbolModel>,
}
impl Model {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_symbols<I: Iterator<Item=String>>(symbols: I) -> Self {
        Self {
            symbols: symbols
                .map(|symbol| (symbol.clone(), SymbolModel::from_symbol(symbol)))
                .collect()
        }
    }
    pub async fn get_symbol_model<'a>(&'a self, symbol: String) -> Result<&'a SymbolModel, crate::Error> {
        self.symbols.get(&symbol.to_uppercase())
            .ok_or(crate::Error::from(Error::from(format!("No Model for Symbol: {}", symbol))))
    }
    pub async fn add_symbol(&mut self, symbol: String) -> Result<(), crate::Error> {
        debug!("Adding symbol to model...");
        let symbol = symbol.to_uppercase();
        if self.symbols.contains_key(&symbol) {
            Err(Error::from(format!("Symbol {} already in Model", symbol)).into())
        } else if !crate::binance().await.symbol_available(&symbol).await {
            Err(Error::from(format!("Symbol {} not found on Binance", symbol)).into())
        } else {
            self.symbols.insert(symbol.clone(), SymbolModel::from_symbol(symbol));
            Ok(())
        }
    }
    pub async fn update(&mut self) -> Result<(), crate::Error>{
        debug!("Model update");
        for (_, model) in &mut self.symbols {
            model.update().await?
        }
        Ok(())
    }
}
