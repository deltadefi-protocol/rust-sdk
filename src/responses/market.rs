use serde::{Deserialize, Serialize};

/// Represents the market depth (price and quantity).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketDepth {
    pub price: f64,
    pub quantity: f64,
}

/// Represents the response for getting market depth.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMarketDepthResponse {
    pub bids: Vec<MarketDepth>,
    pub asks: Vec<MarketDepth>,
}

/// Represents the response for getting the market price.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMarketPriceResponse {
    pub price: f64,
}

/// Represents a candlestick data point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Candlestick {
    #[serde(rename = "t")]
    pub timestamp: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
}

/// Represents a trade in the aggregated price response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    pub time: String,
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Represents the response for getting aggregated price data.
pub type GetAggregatedPriceResponse = Vec<Candlestick>;
