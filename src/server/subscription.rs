use crate::shared::PriceHistoryRequest;
use crate::Error;
use app_model::market::PriceHistory;
use openlimits::model::Paginator;

#[derive(Debug, Clone)]
pub struct PriceSubscription {
    market_pair: String,
    time_interval: openlimits::model::Interval,
    last_update: Option<chrono::DateTime<chrono::Utc>>,
}
impl From<String> for PriceSubscription {
    fn from(market_pair: String) -> Self {
        Self {
            market_pair,
            time_interval: openlimits::model::Interval::OneMinute,
            last_update: None,
        }
    }
}
impl PriceSubscription {
    pub async fn latest_price_history(&self) -> Result<PriceHistory, Error> {
        let paginator = self.last_update.map(|datetime| {
            Paginator {
                start_time: Some(datetime.timestamp_millis() as u64),
                ..Default::default()
            }
        });
        let req = PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.time_interval),
            paginator,
        };
        let candles = crate::binance().await.get_symbol_price_history(req).await?;
        Ok(PriceHistory {
            market_pair: self.market_pair.clone(),
            time_interval: self.time_interval,
            candles,
        })
    }
}
