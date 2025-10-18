use crate::model::OrderJSON;
use serde::{Deserialize, Serialize}; // Assuming OrderJSON is defined in the `model` module.

/// Represents the response for building a place order transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildPlaceOrderTransactionResponse {
    pub order_id: String,
    pub tx_hex: String,
}

/// Represents the response for submitting a place order transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitPlaceOrderTransactionResponse {
    pub order: OrderJSON,
}

/// Represents the response for building a cancel order transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildCancelOrderTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for building a cancel all orders transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildCancelAllOrdersTransactionResponse {
    pub tx_hexes: Vec<String>,
}

/// Represents the response for submitting a cancel all orders transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitCancelAllOrdersTransactionResponse {
    pub cancelled_order_ids: Vec<String>,
}
