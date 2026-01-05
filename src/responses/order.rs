//! Order Response Types
//!
//! This module contains response types for order-related API operations.

use crate::model::Order;
use serde::{Deserialize, Serialize};

/// Represents the response for building a place order transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildPlaceOrderTransactionResponse {
    pub order_id: String,
    pub tx_hex: String,
}

/// Represents the response for submitting a place order transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitPlaceOrderTransactionResponse {
    pub order: Order,
}

/// Represents the response for cancelling a single order.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelOrderResponse {
    pub order_id: String,
}

/// Represents the response for cancelling all orders.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelAllOrdersResponse {
    pub symbol: String,
    pub order_ids: Vec<String>,
}
