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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// ADA to USDM trading pair
    #[serde(rename = "ADAUSDM")]
    ADAUSDM,
    /// HOSKY to USDM trading pair
    #[serde(rename = "HOSKYUSDM")]
    HOSKYUSDM,
    /// NIGHT to USDM trading pair
    #[serde(rename = "NIGHTUSDM")]
    NIGHTUSDM,
    /// IAG to USDM trading pair
    #[serde(rename = "IAGUSDM")]
    IAGUSDM,
    /// SNEK to USDM trading pair
    #[serde(rename = "SNEKUSDM")]
    SNEKUSDM,
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
    /// Order transaction is being built
    Building,
    /// Order is being processed by the system
    Processing,
    /// Order is active and waiting to be matched
    Open,
    /// Order has been cancelled
    Cancelled,
    /// Order execution failed due to an error
    Failed,
    /// Order has been closed (terminal state, fully executed)
    Closed,
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

/// Transfer status for transferal records.
///
/// Represents the status of a transfer operation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    /// Transfer is pending confirmation
    Pending,
    /// Transfer has been confirmed on the blockchain
    Confirmed,
}

/// Direction of a transfer.
///
/// Indicates whether the transfer is incoming or outgoing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    /// Incoming transfer (receiving assets)
    Incoming,
    /// Outgoing transfer (sending assets)
    Outgoing,
}

/// Type of transferal operation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferalType {
    /// Direct transfer
    Direct,
    /// Request-based transfer
    Request,
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
pub struct OrderExecutionRecord {
    /// Unique execution identifier
    pub id: String,
    /// ID of the order that was executed
    pub order_id: String,
    /// Account ID of the order owner
    pub account_id: String,
    /// Price at which the execution occurred
    pub execution_price: String,
    /// Base quantity filled in this execution
    pub filled_base_qty: String,
    /// Quote quantity filled in this execution
    pub filled_quote_qty: String,
    /// Unit of the commission charged (e.g., "ADA", "USDM")
    pub commission_unit: String,
    /// Commission amount charged for this execution
    pub commission: String,
    /// Role in this execution (maker or taker)
    pub role: OrderExecutionRole,
    /// Order ID of the counterparty in this execution
    pub counter_party_order_id: String,
    /// Timestamp when this execution occurred (ISO 8601 format)
    pub created_at: String,
}

/// Represents an order response from the API.
///
/// This struct matches the API's `OrderResponse` format exactly.
/// The type alias `OrderResponse` is provided for API naming consistency.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    /// Unique order identifier
    pub id: String,
    /// Account ID that owns this order
    pub account_id: String,
    /// Active order UTXO ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_order_utxo_id: Option<String>,
    /// Current order status
    pub status: OrderStatus,
    /// Trading pair symbol
    pub symbol: String,
    /// Base quantity for the order
    pub base_qty: String,
    /// Quote quantity for the order
    pub quote_qty: String,
    /// Order side (buy/sell)
    pub side: OrderSide,
    /// Order price
    pub price: String,
    /// Order type (market/limit)
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Slippage in basis points (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bp: Option<u64>,
    /// Market order limit price (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_order_limit_price: Option<String>,
    /// Locked base quantity
    pub locked_base_qty: String,
    /// Locked quote quantity
    pub locked_quote_qty: String,
    /// Executed base quantity
    pub executed_base_qty: String,
    /// Executed quote quantity
    pub executed_quote_qty: String,
    /// Open order base quantity in orderbook
    pub ob_open_order_base_qty: String,
    /// Commission unit
    pub commission_unit: String,
    /// Total commission charged
    pub commission: String,
    /// Commission rate in basis points
    pub commission_rate_bp: u64,
    /// Executed price
    pub executed_price: String,
    /// Order creation timestamp (ISO 8601 format)
    pub created_at: String,
    /// Order last update timestamp (ISO 8601 format)
    pub updated_at: String,
    /// Order execution records (fills)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_execution_records: Option<Vec<OrderExecutionRecord>>,
}

/// Type alias for Order to match API naming convention.
pub type OrderResponse = Order;

/// Type alias for OrderExecutionRecord to match API naming convention.
pub type OrderExecutionRecordResponse = OrderExecutionRecord;

/// Represents an order execution record in JSON format returned by the API.
#[deprecated(since = "0.2.0", note = "Use OrderExecutionRecord instead")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderExecutionRecordJSON {
    pub id: String,
    pub order_id: String,
    pub execution_price: String,
    pub filled_amount: String,
    pub fee_unit: String,
    pub fee_amount: String,
    pub role: OrderExecutionRole,
    pub counter_party_order_id: String,
    pub create_time: u64,
}

/// Represents an order filling record in JSON format (legacy compatibility).
#[deprecated(since = "0.2.0", note = "Use OrderExecutionRecord instead")]
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

/// Order format returned by the API.
#[deprecated(since = "0.2.0", note = "Use Order (or OrderResponse type alias) instead")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderJSON {
    pub order_id: String,
    pub status: String,
    pub symbol: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub side: OrderSide,
    pub price: f64,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub fee_charged: String,
    pub fee_unit: String,
    pub executed_price: f64,
    pub slippage: String,
    pub create_time: u64,
    pub update_time: u64,
    #[serde(default)]
    pub fills: Vec<OrderExecutionRecordJSON>,
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

/// Represents a transferal record.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferalRecord {
    pub created_at: String,
    pub status: TransferStatus,
    pub assets: Vec<Asset>,
    pub transferal_type: TransferalType,
    pub tx_hash: String,
    pub direction: TransferDirection,
}

/// Represents an asset balance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetBalance {
    pub asset: String,
    pub asset_unit: String,
    /// Free balance as decimal string for precision
    pub free: String,
    /// Locked balance as decimal string for precision
    pub locked: String,
}

/// Represents an asset with quantity.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub asset: String,
    pub asset_unit: String,
    /// Quantity as decimal string for precision
    pub qty: String,
}

/// Generic paginated response wrapper matching API format.
///
/// Used for endpoints that return paginated lists of items.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginatedResponse<T> {
    /// The data items for the current page
    pub data: Vec<T>,
    /// Total count of items across all pages
    pub total_count: i64,
    /// Total number of pages
    pub total_page: i64,
}
