use serde::{Deserialize, Serialize};

/// Represents trading symbols.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Symbol {
    #[serde(rename = "ADAUSDM")]
    ADAUSDM,
}

/// Represents time intervals for aggregated price data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Interval {
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "1d")]
    Interval1d,
}

/// Represents the possible statuses of an order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Processing,
    Open,
    FullyFilled,
    PartiallyFilled,
    Cancelled,
    PartiallyCancelled,
    Failed,
}

/// Represents the possible sides of an order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Represents the possible types of an order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

/// Represents the transaction statuses.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Building,
    HeldForOrder,
    Submitted,
    SubmissionFailed,
    Confirmed,
}

/// Represents the role in an order execution.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderExecutionRole {
    Maker,
    Taker,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderRecordStatus {
    OpenOrder,
    OrderHistory,
    TradingHistory,
}

/// Represents an order execution record in JSON format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderExecutionRecordJSON {
    pub id: String,
    pub order_id: String,
    pub execution_price: f64,
    pub filled_amount: String,
    pub fee_unit: String,
    pub fee_amount: String,
    pub role: OrderExecutionRole,
    pub counter_party_order_id: String,
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
