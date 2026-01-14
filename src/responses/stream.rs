//! Stream Response Types
//!
//! This module contains response types for WebSocket stream messages.
//! These types represent real-time updates pushed through account and market streams.

use crate::model::{AssetBalance, Order};
use serde::{Deserialize, Serialize};

// ============================================================================
// Market Stream Types
// ============================================================================

/// Price level in the order book (bid or ask).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceLevel {
    /// Price at this level
    pub price: f64,
    /// Total quantity at this price level
    pub quantity: f64,
}

/// Market depth snapshot message.
///
/// Contains the current state of the order book with bids and asks.
/// Received from the `/market/ws/depth/{symbol}` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketDepthMessage {
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Bid orders (buy side) sorted by price descending
    pub bids: Vec<PriceLevel>,
    /// Ask orders (sell side) sorted by price ascending
    pub asks: Vec<PriceLevel>,
}

/// Market price change message.
///
/// Sent when the market price changes due to a trade.
/// Received from the `/market/ws/market-price/{symbol}` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketPriceMessage {
    /// Current market price
    pub price: f64,
}

/// Recent trade data.
///
/// Represents a single executed trade in the market.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    /// Order ID associated with this trade
    pub order_id: String,
    /// Timestamp of the trade (ISO 8601 format)
    pub timestamp: String,
    /// Trading pair symbol (e.g., "ADAUSDM")
    pub symbol: String,
    /// Execution price
    pub price: f64,
    /// Trade amount/quantity
    pub amount: f64,
    /// Trade side ("buy" or "sell")
    pub side: String,
}

/// OHLC (candlestick) data message.
///
/// Contains Open, High, Low, Close, Volume data for a time period.
/// Received from the `/market/ws/graph/{symbol}` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OhlcMessage {
    /// Timestamp (Unix seconds)
    #[serde(rename = "t")]
    pub timestamp: i32,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Open price
    #[serde(rename = "o")]
    pub open: f64,
    /// High price
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price
    #[serde(rename = "c")]
    pub close: f64,
    /// Volume
    #[serde(rename = "v")]
    pub volume: f64,
}

/// Unified market stream message enum.
///
/// Represents all possible messages from market data streams.
#[derive(Debug, Clone)]
pub enum MarketStreamMessage {
    /// Order book depth snapshot
    Depth(MarketDepthMessage),
    /// Market price update
    Price(MarketPriceMessage),
    /// Recent trades (array of trades)
    RecentTrades(Vec<Trade>),
    /// OHLC candlestick data
    Ohlc(OhlcMessage),
    /// Unknown or unparseable message
    Unknown(String),
}

impl MarketStreamMessage {
    /// Returns true if this is a depth message.
    pub fn is_depth(&self) -> bool {
        matches!(self, MarketStreamMessage::Depth(_))
    }

    /// Returns true if this is a price message.
    pub fn is_price(&self) -> bool {
        matches!(self, MarketStreamMessage::Price(_))
    }

    /// Returns true if this is a recent trades message.
    pub fn is_recent_trades(&self) -> bool {
        matches!(self, MarketStreamMessage::RecentTrades(_))
    }

    /// Returns true if this is an OHLC message.
    pub fn is_ohlc(&self) -> bool {
        matches!(self, MarketStreamMessage::Ohlc(_))
    }

    /// Returns true if this is an unknown message.
    pub fn is_unknown(&self) -> bool {
        matches!(self, MarketStreamMessage::Unknown(_))
    }

    /// Try to get the depth message.
    pub fn as_depth(&self) -> Option<&MarketDepthMessage> {
        match self {
            MarketStreamMessage::Depth(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get the price message.
    pub fn as_price(&self) -> Option<&MarketPriceMessage> {
        match self {
            MarketStreamMessage::Price(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get the recent trades.
    pub fn as_recent_trades(&self) -> Option<&Vec<Trade>> {
        match self {
            MarketStreamMessage::RecentTrades(trades) => Some(trades),
            _ => None,
        }
    }

    /// Try to get the OHLC message.
    pub fn as_ohlc(&self) -> Option<&OhlcMessage> {
        match self {
            MarketStreamMessage::Ohlc(msg) => Some(msg),
            _ => None,
        }
    }
}

// ============================================================================
// Account Stream Types
// ============================================================================

/// Stream message type identifier.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StreamType {
    /// Account-related stream messages
    #[serde(rename = "Account")]
    Account,
}

/// Stream message sub-type identifier.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StreamSubType {
    /// Balance update message
    #[serde(rename = "balance")]
    Balance,
    /// Order info update message
    #[serde(rename = "order_info")]
    OrderInfo,
    /// DLTA points update message
    #[serde(rename = "dlta_points")]
    DltaPoints,
}

/// Base stream message structure containing type information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamMessageBase {
    /// The type of stream message
    #[serde(rename = "type")]
    pub msg_type: StreamType,
    /// The sub-type of stream message
    pub sub_type: StreamSubType,
}

/// Account balance stream message.
///
/// Sent when the user's account balance changes due to trades,
/// deposits, withdrawals, or other operations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountBalanceMessage {
    /// The type of stream message
    #[serde(rename = "type")]
    pub msg_type: StreamType,
    /// The sub-type of stream message
    pub sub_type: StreamSubType,
    /// Updated balance information
    pub balance: Vec<AssetBalance>,
}

/// Order info stream message.
///
/// Sent when an order status changes (created, filled, cancelled, etc.).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoMessage {
    /// The type of stream message
    #[serde(rename = "type")]
    pub msg_type: StreamType,
    /// The sub-type of stream message
    pub sub_type: StreamSubType,
    /// Updated order information
    pub order: Order,
}

/// DLTA points data structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DltaPointsData {
    /// Change in points (positive or negative)
    pub delta: String,
    /// New total points after the change
    pub new_total: String,
    /// Current season points
    pub season_points: String,
    /// Source of the points change (e.g., "trade", "referral")
    pub source_type: String,
    /// Optional reference to the source (e.g., order ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    /// Current league tier
    pub league: String,
}

/// DLTA points stream message.
///
/// Sent when the user's DLTA points balance changes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DltaPointsMessage {
    /// The type of stream message
    #[serde(rename = "type")]
    pub msg_type: StreamType,
    /// The sub-type of stream message
    pub sub_type: StreamSubType,
    /// DLTA points update data
    pub dlta_points: DltaPointsData,
}

/// Unified stream message enum for type-safe message handling.
///
/// This enum represents all possible messages that can be received
/// from the account stream WebSocket connection.
///
/// # Example
///
/// ```rust,ignore
/// use deltadefi::StreamMessage;
///
/// match message {
///     StreamMessage::Balance(msg) => {
///         println!("Balance updated: {:?}", msg.balance);
///     }
///     StreamMessage::OrderInfo(msg) => {
///         println!("Order updated: {} - {:?}", msg.order.id, msg.order.status);
///     }
///     StreamMessage::DltaPoints(msg) => {
///         println!("Points changed by {}", msg.dlta_points.delta);
///     }
///     StreamMessage::Unknown(raw) => {
///         println!("Unknown message: {}", raw);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum StreamMessage {
    /// Account balance update
    Balance(AccountBalanceMessage),
    /// Order information update
    OrderInfo(OrderInfoMessage),
    /// DLTA points update
    DltaPoints(DltaPointsMessage),
    /// Unknown or unparseable message (contains raw JSON string)
    Unknown(String),
}

impl StreamMessage {
    /// Parse a raw JSON string into a StreamMessage.
    ///
    /// This method attempts to parse the message based on the `sub_type` field.
    /// If parsing fails or the message type is unknown, it returns `StreamMessage::Unknown`.
    pub fn from_json(json: &str) -> Self {
        // First, try to parse the base message to determine the type
        let base: Result<StreamMessageBase, _> = serde_json::from_str(json);

        match base {
            Ok(base) => match base.sub_type {
                StreamSubType::Balance => {
                    match serde_json::from_str::<AccountBalanceMessage>(json) {
                        Ok(msg) => StreamMessage::Balance(msg),
                        Err(_) => StreamMessage::Unknown(json.to_string()),
                    }
                }
                StreamSubType::OrderInfo => {
                    match serde_json::from_str::<OrderInfoMessage>(json) {
                        Ok(msg) => StreamMessage::OrderInfo(msg),
                        Err(_) => StreamMessage::Unknown(json.to_string()),
                    }
                }
                StreamSubType::DltaPoints => {
                    match serde_json::from_str::<DltaPointsMessage>(json) {
                        Ok(msg) => StreamMessage::DltaPoints(msg),
                        Err(_) => StreamMessage::Unknown(json.to_string()),
                    }
                }
            },
            Err(_) => StreamMessage::Unknown(json.to_string()),
        }
    }

    /// Returns true if this is a balance update message.
    pub fn is_balance(&self) -> bool {
        matches!(self, StreamMessage::Balance(_))
    }

    /// Returns true if this is an order info message.
    pub fn is_order_info(&self) -> bool {
        matches!(self, StreamMessage::OrderInfo(_))
    }

    /// Returns true if this is a DLTA points message.
    pub fn is_dlta_points(&self) -> bool {
        matches!(self, StreamMessage::DltaPoints(_))
    }

    /// Returns true if this is an unknown message.
    pub fn is_unknown(&self) -> bool {
        matches!(self, StreamMessage::Unknown(_))
    }

    /// Try to get the balance message, if this is a balance update.
    pub fn as_balance(&self) -> Option<&AccountBalanceMessage> {
        match self {
            StreamMessage::Balance(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get the order info message, if this is an order update.
    pub fn as_order_info(&self) -> Option<&OrderInfoMessage> {
        match self {
            StreamMessage::OrderInfo(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get the DLTA points message, if this is a points update.
    pub fn as_dlta_points(&self) -> Option<&DltaPointsMessage> {
        match self {
            StreamMessage::DltaPoints(msg) => Some(msg),
            _ => None,
        }
    }
}
