use lazy_static::lazy_static;
use openlimits::{
    binance::{
        model::{
            KlineSummary,
            KlineSummaries,
        },
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
#[derive(Default)]
pub struct SymbolModel {
    symbol: String,
    klines: Vec<KlineSummary>,
}

impl SymbolModel {
    pub fn from_symbol(symbol: String) -> Self {
        Self{
            symbol,
            klines: Vec::new(),
        }
    }
    pub async fn update(&mut self) -> Result<(), crate::Error> {
        self.klines = match crate::binance().await.get_symbol_price_history(&self.symbol).await? {
            KlineSummaries::AllKlineSummaries(v) => v,
        };
        println!("{}: {}", self.symbol, self.klines.last()
            .map(|summary| summary.close.to_string())
            .unwrap_or("-".to_string()));
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
    pub async fn add_symbol(&mut self, symbol: String) -> Result<(), crate::Error> {
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
        for (_, model) in &mut self.symbols {
            model.update().await?
        }
        Ok(())
    }
}
