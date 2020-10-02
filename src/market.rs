use serde::{
    Serialize,
    Deserialize,
};
use openlimits::{
    model::{
        Candle,
        Interval,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistory {
    pub market_pair: String,
    pub candles: Vec<Candle>,
    pub time_interval: Interval,
}
