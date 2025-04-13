use serde::{Deserialize, Serialize};

/// Represents the possible statuses of an order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Building,
    Open,
    Closed,
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

/// Represents an order in JSON format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderJSON {
    pub order_id: String,
    pub status: OrderStatus,
    pub symbol: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub side: OrderSide,
    pub price: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub fee_amount: f64,
    pub executed_price: f64,
    pub slippage: String,
    pub create_time: i64,
    pub update_time: i64,
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
    pub free: i64,
    pub locked: i64,
}

/// Represents an asset (placeholder for the actual definition).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub amount: i64,
}
