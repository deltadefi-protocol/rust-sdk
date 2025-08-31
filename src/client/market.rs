use super::Api;
use crate::{responses::market::*, Symbol, Interval};
use serde_json::from_str;
use whisky::WError;

pub struct Market {
    pub api: Api,
    pub path_url: String,
}

impl Market {
    pub fn new(api: Api) -> Self {
        Market {
            api,
            path_url: "/market".to_string(),
        }
    }

    /// Retrieves the market price.
    pub async fn get_market_price(&self, symbol: &str) -> Result<GetMarketPriceResponse, WError> {
        let url = format!("{}/market-price?symbol={}", self.path_url, symbol);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_market_price"))?)
    }

    /// Retrieves the aggregated price data.
    pub async fn get_aggregated_price(
        &self,
        symbol: Symbol,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<GetAggregatedPriceResponse, WError> {
        let symbol_str = match symbol {
            Symbol::ADAUSDM => "ADAUSDM",
        };
        let interval_str = match interval {
            Interval::Interval5m => "5m",
            Interval::Interval15m => "15m",
            Interval::Interval30m => "30m",
            Interval::Interval1h => "1h",
            Interval::Interval1d => "1d",
        };

        let url = format!(
            "{}/graph/{}?interval={}&start={}&end={}",
            self.path_url, symbol_str, interval_str, start, end
        );
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_aggregated_price"))?)
    }
}
