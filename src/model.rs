//! Data Models and Type Definitions
//!
//! This module contains all the core data types, enums, and structures used throughout
//! the DeltaDeFi SDK. These types provide strong typing for API parameters and responses,
//! ensuring type safety and better developer experience.

use serde::{Deserialize, Serialize};

/// Trading symbols supported by the DeltaDeFi platform.
///
/// Represents the available trading pairs that can be used for placing orders
/// and retrieving market data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Symbol {
    /// ADA to USDM trading pair
    #[serde(rename = "ADAUSDM")]
    ADAUSDM,
}

/// Time intervals for aggregated market data.
///
/// Used when retrieving historical price data to specify the granularity
/// of the data points. Each interval represents the time period over which
/// price data is aggregated.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Interval {
    /// 5-minute intervals
    #[serde(rename = "5m")]
    Interval5m,
    /// 15-minute intervals
    #[serde(rename = "15m")]
    Interval15m,
    /// 30-minute intervals
    #[serde(rename = "30m")]
    Interval30m,
    /// 1-hour intervals
    #[serde(rename = "1h")]
    Interval1h,
    /// 1-day intervals
    #[serde(rename = "1d")]
    Interval1d,
}

/// Order execution status in the DeltaDeFi system.
///
/// Represents the various states an order can be in during its lifecycle,
/// from initial processing to final completion or cancellation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order is being processed by the system
    Processing,
    /// Order is active and waiting to be matched
    Open,
    /// Order has been completely executed
    FullyFilled,
    /// Order has been partially executed with remaining quantity open
    PartiallyFilled,
    /// Order has been cancelled before any execution
    Cancelled,
    /// Order was partially filled and then cancelled
    PartiallyCancelled,
    /// Order execution failed due to an error
    Failed,
}

/// Order side indicating buy or sell direction.
///
/// Specifies whether the order is purchasing (buy) or selling (sell) the base asset
/// in the trading pair.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    /// Purchase order - buying the base asset
    Buy,
    /// Sell order - selling the base asset
    Sell,
}

/// Order execution type determining price and execution behavior.
///
/// Defines how the order should be executed in terms of pricing and timing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    /// Market order - executes immediately at current market price
    Market,
    /// Limit order - executes only at specified price or better
    Limit,
}

/// Transaction processing status in the DeltaDeFi system.
///
/// Tracks the lifecycle of transactions from creation to final confirmation
/// on the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Transaction is being constructed
    Building,
    /// Transaction is held pending order matching
    HeldForOrder,
    /// Transaction has been submitted to the blockchain
    Submitted,
    /// Transaction submission to blockchain failed
    SubmissionFailed,
    /// Transaction has been confirmed on the blockchain
    Confirmed,
}

/// Role of a participant in an order execution.
///
/// In order matching, one party provides liquidity (maker) while the other
/// takes liquidity (taker) from the order book.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderExecutionRole {
    /// Liquidity provider - order was in the book first
    Maker,
    /// Liquidity taker - order matched against existing book order
    Taker,
}

/// Filter for querying different types of order records.
///
/// Used when retrieving order records to specify which category of orders
/// to include in the response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderRecordStatus {
    /// Currently active/open orders
    OpenOrder,
    /// Historical order data (completed/cancelled)
    OrderHistory,
    /// Trading execution history
    TradingHistory,
}

/// Order execution record containing details of a trade execution.
///
/// Represents a single trade execution that occurred when an order was matched.
/// An order may have multiple execution records if it was filled in multiple trades.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderExecutionRecordJSON {
    /// Unique execution identifier
    pub id: String,
    /// ID of the order that was executed
    pub order_id: String,
    /// Price at which the execution occurred
    pub execution_price: f64,
    /// Amount filled in this execution
    pub filled_amount: String,
    /// Unit of the fee charged (e.g., "ADA", "USDM")
    pub fee_unit: String,
    /// Fee amount charged for this execution
    pub fee_amount: String,
    /// Role in this execution (maker or taker)
    pub role: OrderExecutionRole,
    /// Order ID of the counterparty in this execution
    pub counter_party_order_id: String,
    /// Timestamp when this execution occurred (Unix timestamp)
    pub create_time: i64,
}

/// Represents an order filling record in JSON format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderFillingRecordJSON {
    pub execution_id: String,
    pub order_id: String,
    pub status: String,
    pub symbol: String,
    pub executed_qty: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub fee_charged: String,
    pub fee_unit: String,
    pub executed_price: f64,
    pub create_time: u64,
}

/// Represents an order in JSON format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderJSON {
    pub order_id: String,
    pub status: OrderStatus,
    pub symbol: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub side: OrderSide,
    pub price: f64,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub fee_charged: String,
    pub executed_price: f64,
    pub slippage: String,
    pub create_time: i64,
    pub update_time: i64,
    pub fills: Option<Vec<OrderExecutionRecordJSON>>,
}

/// Represents a deposit record.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositRecord {
    pub created_at: String,
    pub status: TransactionStatus,
    pub assets: Vec<Asset>,
    pub tx_hash: String,
}

/// Represents a withdrawal record.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WithdrawalRecord {
    pub created_at: String,
    pub status: TransactionStatus,
    pub assets: Vec<Asset>,
}

/// Represents an asset balance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetBalance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

/// Represents an asset (placeholder for the actual definition).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub asset: String,
    pub asset_unit: String,
    pub qty: f64,
}
