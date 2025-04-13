use super::Api;
use crate::responses::market::*;
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

    /// Retrieves the market depth.
    pub async fn get_depth(&self, symbol: &str) -> Result<GetMarketDepthResponse, WError> {
        let url = format!("{}/depth?symbol={}", self.path_url, symbol);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_depth"))?)
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
        symbol: &str,
        interval: &str,
        start: u64,
        end: u64,
    ) -> Result<GetAggregatedPriceResponse, WError> {
        let url = format!(
            "{}/aggregated-trade/{}?interval={}&start={}&end={}",
            self.path_url, symbol, interval, start, end
        );
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_aggregated_price"))?)
    }
}
