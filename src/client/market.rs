//! Market Data Module
//!
//! This module provides functionality for accessing market data from DeltaDeFi, including:
//! - Current market prices
//! - Historical price data with multiple time intervals
//! - Aggregated price information for technical analysis

use super::Api;
use crate::{responses::market::*, Symbol, Interval};
use serde_json::from_str;
use whisky::WError;

/// Client for market data operations on the DeltaDeFi platform.
///
/// Provides methods for retrieving current and historical market data
/// for supported trading pairs. All market data is publicly available
/// and doesn't require special authentication beyond the API key.
pub struct Market {
    /// Internal API client
    pub api: Api,
    /// Base path for market endpoints
    pub path_url: String,
}

impl Market {
    pub fn new(api: Api) -> Self {
        Market {
            api,
            path_url: "/market".to_string(),
        }
    }

    /// Retrieves the current market price for a trading pair.
    ///
    /// Returns the latest market price information including bid, ask, and last trade price
    /// for the specified trading pair symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol (e.g., "ADAUSDM")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the current market price data, or a `WError` if the request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let price_data = client.market.get_market_price("ADAUSDM").await?;
    /// println!("Current price: {}", price_data.price);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Invalid trading pair symbol
    /// - Network request fails
    /// - Market data is temporarily unavailable
    pub async fn get_market_price(&self, symbol: &str) -> Result<GetMarketPriceResponse, WError> {
        let url = format!("{}/market-price?symbol={}", self.path_url, symbol);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_market_price"))?)
    }

    /// Retrieves historical aggregated price data for technical analysis.
    ///
    /// Returns OHLCV (Open, High, Low, Close, Volume) data for the specified trading pair
    /// within the given time range and interval. This data is useful for creating charts
    /// and performing technical analysis.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol from the `Symbol` enum
    /// * `interval` - Time interval for data aggregation (5m, 15m, 30m, 1h, 1d)
    /// * `start` - Start timestamp (Unix timestamp in seconds)
    /// * `end` - End timestamp (Unix timestamp in seconds)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing historical price data points, or a `WError` if the request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use deltadefi::{Symbol, Interval};
    /// use chrono::{Utc, Duration};
    /// 
    /// let now = Utc::now();
    /// let yesterday = now - Duration::hours(24);
    /// 
    /// let historical_data = client.market.get_aggregated_price(
    ///     Symbol::ADAUSDM,
    ///     Interval::Interval1h,
    ///     yesterday.timestamp(),
    ///     now.timestamp(),
    /// ).await?;
    /// 
    /// println!("Data points: {}", historical_data.data.len());
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Invalid symbol or interval
    /// - Invalid time range (start > end)
    /// - Time range is too large
    /// - Network request fails
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
