use crate::shared;
use serde::{
    Serialize,
    Deserialize,
};
use openlimits::{
    errors::{
        OpenLimitError,
    },
    binance::{
        Binance as Api,
    },
    model::{
        GetPriceTickerRequest,
        GetHistoricRatesRequest,
        Ticker,
        Interval,
        Candle
    },
    exchange::{
        OpenLimits,
    },
};
use async_std::sync::{
    Arc,
    Mutex,
};
use lazy_static::lazy_static;
use crate::{
    Error,
};

lazy_static! {
    pub static ref BINANCE: Arc<Mutex<Binance>> = Arc::new(Mutex::new(Binance::new()));
}
#[derive(Serialize, Deserialize)]
pub struct BinanceCredential {
    secret_key: String,
    api_key: String,
}
impl BinanceCredential {
    pub fn new() -> Self {
        Self {
            api_key: shared::read_key_file("keys/binance_api"),
            secret_key: shared::read_key_file("keys/binance_secret"),
        }
    }
}

pub struct Binance {
    api: Option<OpenLimits<Api>>,
}

impl Binance {
    pub fn new() -> Self {
        Self {
            api: None,
        }
    }
    pub async fn init(&mut self) {
        let credential = BinanceCredential::new();
        let api = Api::with_credential(&credential.api_key, &credential.secret_key, false).await;
        self.api = Some(OpenLimits::new(api));
    }
    fn api<'a>(&'a self) -> Result<&'a OpenLimits<Api>, Error> {
        self.api.as_ref().ok_or(Error::from(OpenLimitError::NoApiKeySet()))
    }
    fn api_mut<'a>(&'a mut self) -> Result<&'a mut OpenLimits<Api>, Error> {
        self.api.as_mut().ok_or(Error::from(OpenLimitError::NoApiKeySet()))
    }
    pub async fn get_symbol_price(&self, symbol: &str) -> Result<Ticker, Error> {
        self.api()?.get_price_ticker(&GetPriceTickerRequest {
            market_pair: symbol.to_string().to_uppercase(),
            ..Default::default()
        }).await.map_err(Into::into)
    }
    pub async fn symbol_available(&self, symbol: &str) -> bool {
        self.get_symbol_price(symbol).await.is_ok()
    }
    pub async fn get_symbol_price_history(&self, req: PriceHistoryRequest) -> Result<Vec<Candle>, Error> {
        self.api()?.get_historic_rates(&GetHistoricRatesRequest {
            market_pair: req.market_pair.to_uppercase(),
            interval: req.interval.unwrap_or(Interval::OneMinute),
            paginator: req.paginator,
        }).await.map_err(Into::into)
    }
}
#[derive(Debug)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<openlimits::model::Interval>,
    pub paginator: Option<openlimits::model::Paginator<u64>>,
}


