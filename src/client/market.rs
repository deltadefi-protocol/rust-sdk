use whisky::WError;

use super::Api;

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
    pub async fn get_depth(&self, symbol: &str) -> Result<String, WError> {
        let url = format!("{}/depth?symbol={}", self.path_url, symbol);
        self.api.get(&url).await
    }

    /// Retrieves the market price.
    pub async fn get_market_price(&self, symbol: &str) -> Result<String, WError> {
        let url = format!("{}/market-price?symbol={}", self.path_url, symbol);
        self.api.get(&url).await
    }

    /// Retrieves the aggregated price data.
    pub async fn get_aggregated_price(
        &self,
        pair: &str,
        interval: &str,
        start: u64,
        end: u64,
    ) -> Result<String, WError> {
        let url = format!(
            "{}/aggregate/{}?interval={}&start={}&end={}",
            self.path_url, pair, interval, start, end
        );
        self.api.get(&url).await
    }
}
