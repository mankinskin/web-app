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
use serde::{
    Serialize,
    Deserialize,
};
use tracing::{
    debug,
};
use std::convert::TryInto;
use crate::shared::{
    PriceHistoryRequest,
};
use futures::{
    StreamExt,
};

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
#[derive(Default, Serialize, Deserialize, Clone)]
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
        //debug!("SymbolModel update");
        let paginator = self.last_update.and_then(|date_time|
                date_time.timestamp().try_into().ok()
            ).map(|timestamp: u64| 
                        openlimits::model::Paginator {
                            start_time: Some(timestamp),
                            ..Default::default()
                        }
                    );
        let prices = crate::binance().await.get_symbol_price_history(
                PriceHistoryRequest {
                    market_pair: self.symbol.clone(),
                    interval: None,
                    paginator,
                }
            ).await?;
        self.prices = prices;
        self.last_update = Some(Utc::now());
        Ok(())
    }
    pub async fn is_available(&self) -> bool {
        let symbol = self.symbol.to_uppercase();
        crate::binance().await.symbol_available(&symbol).await
    }
}
#[derive(Default)]
pub struct Model {
    symbols: HashMap<String, SymbolModel>,
    new_symbols: bool,
}
impl Model {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_symbols<I: Iterator<Item=String>>(symbols: I) -> Self {
        Self {
            symbols: symbols
                .map(|symbol| (symbol.clone(), SymbolModel::from_symbol(symbol)))
                .collect(),
            new_symbols: true,
        }
    }
    pub async fn filter_available_symbols(&mut self) -> Result<(), crate::Error> {
        let mut errors = Vec::new();
        self.symbols = futures::stream::iter(self.symbols.clone().into_iter())
            .then(async move |(symbol, model)|
                if model.is_available().await {
                    Ok((symbol, model))
                } else {
                    Err(Error::from(format!("Symbol {} does not exist on binance.", symbol)))
                }
            )
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|result: Result<(String, SymbolModel), Error>| {
                match result {
                    Ok(pair) => Some(pair),
                    Err(error) => { errors.push(error); None }
                }
            })
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::from(errors))
        }
    }
    pub async fn get_symbol_model<'a>(&'a self, symbol: String) -> Result<&'a SymbolModel, crate::Error> {
        self.symbols.get(&symbol.to_uppercase())
            .ok_or(crate::Error::from(Error::from(format!("No Model for Symbol: {}", symbol))))
    }
    pub async fn add_symbol(&mut self, symbol: String) -> Result<(), crate::Error> {
        debug!("Adding symbol to model...");
        if !self.symbols.contains_key(&symbol) {
            self.symbols.insert(symbol.clone(), SymbolModel::from_symbol(symbol));
            self.new_symbols = true;
        } else {
            debug!("Model already exists.");
        }
        Ok(())
    }
    pub async fn update(&mut self) -> Result<(), crate::Error>{
        //debug!("Model update");
        if self.new_symbols {
            self.filter_available_symbols().await?;
            self.new_symbols = false;
        }
        for (_, model) in &mut self.symbols {
            model.update().await?
        }
        Ok(())
    }
}
