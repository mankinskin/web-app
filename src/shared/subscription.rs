use crate::shared::PriceHistoryRequest;
use openlimits::model::{
    Paginator,
    Interval,
};
#[cfg(not(target_arch = "wasm32"))]
use openlimits::model::{
    Candle,
};
use serde::{
    Deserialize,
    Serialize,
};

#[cfg(not(target_arch = "wasm32"))]
use {
    app_model::market::PriceHistory,
    tracing::error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceSubscription {
    pub market_pair: String,
    pub time_interval: Interval,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}
impl From<PriceHistoryRequest> for PriceSubscription {
    fn from(request: PriceHistoryRequest) -> Self {
        Self {
            market_pair: request.market_pair,
            time_interval: request.interval.unwrap_or(Interval::OneMinute),
            last_update: None,
        }
    }
}
impl PartialEq<PriceHistoryRequest> for PriceSubscription {
    fn eq(&self, rhs: &PriceHistoryRequest) -> bool {
        self.market_pair == rhs.market_pair
    }
}
impl PriceSubscription {
    pub fn paginator(&self) -> Option<Paginator<u32>> {
        self.last_update.map(|datetime| {
            Paginator {
                start_time: Some(datetime.timestamp_millis() as u64),
                ..Default::default()
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn is_available(&self) -> bool {
        let symbol = self.market_pair.to_uppercase();
        crate::binance::Binance::symbol_available(&symbol).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_price_history_request(&self) -> PriceHistoryRequest {
        let paginator = self.paginator();
        PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.time_interval),
            paginator,
        }
    }
    //#[cfg(not(target_arch = "wasm32"))]
    //pub async fn latest_price_candles(&self) -> Result<Vec<Candle>, crate::subscriptions::Error> {
    //    let req = self.get_price_history_request().await;
    //    crate::binance().await.get_symbol_price_history(req).await.map_err(|e| e.to_string().into())
    //}
    //#[cfg(not(target_arch = "wasm32"))]
    //pub async fn latest_price_history(&self) -> Result<PriceHistory, crate::binance::Error> {
    //    let candles = self.latest_price_candles().await?;
    //    Ok(PriceHistory {
    //        market_pair: self.market_pair.clone(),
    //        time_interval: self.time_interval,
    //        candles,
    //    })
    //}
}
