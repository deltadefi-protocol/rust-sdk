//! Order Management Module
//!
//! This module provides functionality for managing orders on the DeltaDeFi platform, including:
//! - Building and submitting place order transactions
//! - Building and submitting cancel order transactions
//! - Order validation and parameter checking
//! - Support for both market and limit orders with slippage protection

use serde_json::json;
use whisky::WError;

use crate::{
    order::{
        BuildCancelAllOrdersTransactionResponse, BuildCancelOrderTransactionResponse,
        BuildPlaceOrderTransactionResponse, SubmitCancelAllOrdersTransactionResponse,
        SubmitPlaceOrderTransactionResponse,
    },
    OrderSide, OrderType,
};

use super::Api;

/// Client for order management operations on the DeltaDeFi platform.
///
/// Provides methods for placing and canceling orders. All order operations
/// require proper authentication and operation key signing for execution.
/// Supports both market and limit orders with comprehensive validation.
pub struct Order {
    /// Internal API client
    pub api: Api,
    /// Base path for order endpoints
    pub path_url: String,
}

impl Order {
    pub fn new(api: Api) -> Self {
        Order {
            api,
            path_url: "/order".to_string(),
        }
    }

    /// Builds an unsigned transaction for placing an order.
    ///
    /// Creates an unsigned transaction that, when signed and submitted, will place
    /// an order on the DeltaDeFi platform. This is the first step in the two-phase
    /// order placement process.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol (e.g., "ADAUSDM")
    /// * `side` - Order side: `OrderSide::Buy` or `OrderSide::Sell`
    /// * `order_type` - Order type: `OrderType::Market` or `OrderType::Limit`
    /// * `quantity` - The amount to trade (must be > 0)
    /// * `price` - Required for limit orders, ignored for market orders
    /// * `limit_slippage` - Whether to enable slippage protection for market orders
    /// * `max_slippage_basis_point` - Maximum acceptable slippage in basis points (e.g., 100 = 1%)
    /// * `post_only` - If true, the order will only be posted to the order book and not executed immediately
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the order ID and unsigned transaction hex, or a `WError` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Build a limit buy order
    /// let build_response = client.order.build_place_order_transaction(
    ///     "ADAUSDM",
    ///     OrderSide::Buy,
    ///     OrderType::Limit,
    ///     100.0,
    ///     Some(1.25),  // Limit price
    ///     None,
    ///     None,
    ///     None,
    /// ).await?;
    ///
    /// // Build a market sell order with slippage protection
    /// let build_response = client.order.build_place_order_transaction(
    ///     "ADAUSDM",
    ///     OrderSide::Sell,
    ///     OrderType::Market,
    ///     50.0,
    ///     None,        // No price for market orders
    ///     Some(true),  // Enable slippage protection
    ///     Some(100),   // Max 1% slippage
    ///     None,
    /// ).await?;
    /// ```
    ///
    /// # Validation
    ///
    /// This method performs comprehensive validation:
    /// - Symbol must not be empty
    /// - Quantity must be greater than 0
    /// - Limit orders must include a price
    /// - Market orders with slippage protection must include max_slippage_basis_point
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Required parameters are missing or invalid
    /// - Insufficient account balance
    /// - Trading pair is not available
    /// - Network request fails
    pub async fn build_place_order_transaction(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        limit_slippage: Option<bool>,
        max_slippage_basis_point: Option<u64>,
        post_only: Option<bool>,
    ) -> Result<BuildPlaceOrderTransactionResponse, WError> {
        // Validate required parameters
        if symbol.is_empty() {
            return Err(WError::new(
                "build_place_order_transaction",
                "Missing required parameter: symbol",
            ));
        }
        if quantity <= 0.0 {
            return Err(WError::new(
                "build_place_order_transaction",
                "Missing required parameter: quantity",
            ));
        }

        // Additional validation for limit orders
        if order_type == OrderType::Limit && price.is_none() {
            return Err(WError::new(
                "build_place_order_transaction",
                "Missing required parameter: price for limit order",
            ));
        }

        // Additional validation for market orders with slippage
        if order_type == OrderType::Market {
            if let Some(true) = limit_slippage {
                if max_slippage_basis_point.is_none() {
                    return Err(WError::new(
                      "build_place_order_transaction",
                      "Missing required parameter: max_slippage_basis_point for market order with slippage",
                  ));
                }
            }
        }

        // Build the payload
        let payload = json!({
            "symbol": symbol,
            "side": side,
            "type": order_type,
            "quantity": quantity,
            "price": price,
            "limit_slippage": limit_slippage,
            "max_slippage_basis_point": max_slippage_basis_point,
            "post_only": post_only,
        });

        // Send the request
        let url = format!("{}/build", self.path_url);
        let response = self.api.post(&url, payload).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("build_place_order_transaction"))?)
    }

    /// Builds a cancel order transaction.
    pub async fn build_cancel_order_transaction(
        &self,
        order_id: &str,
    ) -> Result<BuildCancelOrderTransactionResponse, WError> {
        let url = format!("{}/{}/build", self.path_url, order_id);
        let response = self.api.delete(&url, json!({})).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("build_cancel_order_transaction"))?)
    }

    /// Builds cancel all orders transaction.
    pub async fn build_cancel_all_orders_transaction(
        &self,
    ) -> Result<BuildCancelAllOrdersTransactionResponse, WError> {
        let url = format!("{}/cancel-all/build", self.path_url);
        let response = self.api.delete(&url, json!({})).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("build_cancel_order_transaction"))?)
    }

    /// Submits a place order transaction.
    pub async fn submit_place_order_transaction(
        &self,
        order_id: &str,
        signed_tx: &str,
    ) -> Result<SubmitPlaceOrderTransactionResponse, WError> {
        let payload = json!({
            "order_id": order_id,
            "signed_tx": signed_tx,
        });
        let url = format!("{}/submit", self.path_url);
        let response = self.api.post(&url, payload).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("submit_place_order_transaction"))?)
    }

    /// Submits a cancel order transaction.
    pub async fn submit_cancel_order_transaction(&self, signed_tx: &str) -> Result<(), WError> {
        let url = format!("{}/submit", self.path_url);
        let payload = json!({
            "signed_tx": signed_tx,
        });
        self.api.delete(&url, payload).await?;
        Ok(())
    }
    /// Submits a cancel all orders transaction.
    pub async fn submit_cancel_all_orders_transaction(
        &self,
        signed_txs: &[String],
    ) -> Result<SubmitCancelAllOrdersTransactionResponse, WError> {
        let url = format!("{}/cancel-all/submit", self.path_url);
        let payload = json!({
            "signed_txs": signed_txs,
        });
        let response = self.api.delete(&url, payload).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("submit_cancel_all_orders_transaction"))?)
    }
}
