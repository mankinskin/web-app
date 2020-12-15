use crate::{
    keys,
};
use async_std::sync::{
    Arc,
    Mutex,
};
use lazy_static::lazy_static;
use openlimits::{
    binance::Binance as Api,
    exchange::OpenLimits,
    model::{
        GetHistoricRatesRequest,
        GetPriceTickerRequest,
        Interval,
        Ticker,
        Paginator,
    },
};
use app_model::market::PriceHistory;
use serde::{
    Deserialize,
    Serialize,
};
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
};
use std::fmt::{
    Display,
    Formatter,
    self,
};
#[cfg(feature = "actix_server")]
pub mod actix_actor;
#[cfg(feature = "actix_server")]
pub use actix_actor as actor;

#[derive(Clone, Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self(err)
    }
}
#[derive(Serialize, Deserialize)]
pub struct BinanceCredential {
    secret_key: String,
    api_key: String,
}
impl BinanceCredential {
    pub fn new() -> Self {
        Self {
            api_key: keys::read_key_file("binance_api"),
            secret_key: keys::read_key_file("binance_secret"),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
    pub paginator: Option<Paginator<u32>>,
}


pub type ApiHandle = Arc<OpenLimits<Api>>;
pub type ApiHolder = Arc<Mutex<Option<ApiHandle>>>;
lazy_static! {
    pub static ref BINANCE: StaticBinance = StaticBinance::new();
}
pub async fn binance() -> &'static StaticBinance {
    &BINANCE
}
pub struct StaticBinance {
    api: ApiHolder,
}
impl std::ops::Deref for StaticBinance {
    type Target = ApiHolder;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
impl std::ops::DerefMut for StaticBinance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}
impl StaticBinance {
    pub fn new() -> Self {
        Self {
            api: Arc::new(Mutex::new(None)),
        }
    }
    pub async fn set_api(&mut self, api: Option<ApiHandle>) {
        *self.api.lock().await = api;
    }
    pub async fn get_api(&self) -> Result<ApiHandle, Error>{
        self.api
            .lock().await
            .as_ref()
            .ok_or(String::from("Binance API not initialized!"))
            .map_err(Into::into)
            .map(Clone::clone)
    }
    pub async fn get_symbol_price(&self, symbol: &str) -> Result<Ticker, Error> {
        //debug!("Requesting symbol price...");
        self.get_api().await?.get_price_ticker(&GetPriceTickerRequest {
                market_pair: symbol.to_string().to_uppercase(),
                ..Default::default()
            })
            .await
            .map_err(|e| Error::from(e.to_string()))
    }
    pub async fn symbol_available(&self, symbol: &str) -> bool {
        self.get_symbol_price(symbol).await.is_ok()
    }
    pub async fn get_symbol_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<PriceHistory, Error> {
        //info!("Requesting symbol price history",);
        //debug!("{:#?}", req);
        let time_interval = req.interval.unwrap_or(Interval::OneMinute);
        let market_pair = req.market_pair.to_uppercase();
        self.get_api().await?
            .get_historic_rates(&GetHistoricRatesRequest {
                market_pair: market_pair.clone(),
                interval: time_interval.clone(),
                paginator: req.paginator.map(|p: Paginator<u32>|
                    Paginator {
                        after: p.after.map(|x| x as u64),
                        before: p.before.map(|x| x as u64),
                        start_time: p.start_time,
                        end_time: p.end_time,
                        limit: p.limit,
                    }
                )
            })
            .await
            .map_err(|e| Error::from(e.to_string()))
            .map(|candles| PriceHistory {
                market_pair,
                time_interval,
                candles,
            })
    }
}
