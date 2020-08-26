use crate::shared;
use serde::{
    Serialize,
    Deserialize,
};
use openlimits::{
    shared::{
        Result as OpenLimitResult,
    },
    binance::{
        Binance as Api,
        model::{
            KlineParams,
            KlineSummaries,
            KlineSummary,
        },
    },
    exchange::Exchange,
    model::{
        GetPriceTickerRequest,
        Ticker,
    },
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BINANCE: Binance = Binance::new();
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

#[derive(Clone)]
pub struct Binance {
    api: Api,
}

impl Binance {
    pub fn new() -> Self {
        let credential = BinanceCredential::new();
        let api = Api::with_credential(&credential.api_key, &credential.secret_key, false);
        Self {
            api,
        }
    }
    pub async fn get_symbol_price(&self, symbol: &str) -> OpenLimitResult<Ticker> {
        self.api.get_price_ticker(&GetPriceTickerRequest {
            symbol: symbol.to_string().to_uppercase(),
            ..Default::default()
        }).await
    }
    pub async fn symbol_available(&self, symbol: &str) -> bool {
        self.get_symbol_price(symbol).await.is_ok()
    }
    pub async fn get_symbol_price_history(&self, symbol: &str) -> OpenLimitResult<Vec<KlineSummary>> {
        match self.api.get_klines(&KlineParams {
            symbol: symbol.to_string().to_uppercase(),
            interval: "1m".to_string(),
            paginator: None,
        }).await? {
            KlineSummaries::AllKlineSummaries(v) => Ok(v),
        }
    }
}
