//! Order Management Module
//!
//! This module provides functionality for managing orders on the DeltaDeFi platform, including:
//! - Building and submitting place order transactions
//! - Cancelling single orders or all orders for a symbol
//! - Order validation and parameter checking
//! - Support for both market and limit orders with slippage protection

use serde_json::json;
use whisky::WError;

use crate::{
    order::{
        BuildPlaceOrderTransactionResponse, CancelAllOrdersResponse, CancelOrderResponse,
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
    /// * `base_quantity` - The base asset quantity (mutually exclusive with quote_quantity)
    /// * `quote_quantity` - The quote asset quantity (mutually exclusive with base_quantity)
    /// * `price` - Required for limit orders, ignored for market orders
    /// * `max_slippage_basis_point` - Maximum acceptable slippage in basis points (e.g., "100" = 1%)
    /// * `post_only` - If true, the order will only be posted to the order book and not executed immediately
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the order ID and unsigned transaction hex, or a `WError` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Build a limit buy order with base quantity
    /// let build_response = client.order.build_place_order_transaction(
    ///     "ADAUSDM",
    ///     OrderSide::Buy,
    ///     OrderType::Limit,
    ///     Some("100.0".to_string()),  // base quantity
    ///     None,                        // no quote quantity
    ///     Some("1.25".to_string()),    // Limit price
    ///     None,
    ///     None,
    /// ).await?;
    ///
    /// // Build a market sell order with slippage protection
    /// let build_response = client.order.build_place_order_transaction(
    ///     "ADAUSDM",
    ///     OrderSide::Sell,
    ///     OrderType::Market,
    ///     Some("50.0".to_string()),     // base quantity
    ///     None,
    ///     None,                          // No price for market orders
    ///     Some("100".to_string()),       // Max 1% slippage
    ///     None,
    /// ).await?;
    /// ```
    ///
    /// # Validation
    ///
    /// This method performs comprehensive validation:
    /// - Symbol must not be empty
    /// - Either base_quantity or quote_quantity must be provided (not both)
    /// - Limit orders must include a price
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
        base_quantity: Option<String>,
        quote_quantity: Option<String>,
        price: Option<String>,
        max_slippage_basis_point: Option<String>,
        post_only: Option<bool>,
    ) -> Result<BuildPlaceOrderTransactionResponse, WError> {
        // Validate required parameters
        if symbol.is_empty() {
            return Err(WError::new(
                "build_place_order_transaction",
                "Missing required parameter: symbol",
            ));
        }

        // Validate that exactly one of base_quantity or quote_quantity is provided
        match (&base_quantity, &quote_quantity) {
            (None, None) => {
                return Err(WError::new(
                    "build_place_order_transaction",
                    "Either base_quantity or quote_quantity must be provided",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(WError::new(
                    "build_place_order_transaction",
                    "Cannot provide both base_quantity and quote_quantity",
                ));
            }
            _ => {}
        }

        // Additional validation for limit orders
        if order_type == OrderType::Limit && price.is_none() {
            return Err(WError::new(
                "build_place_order_transaction",
                "Missing required parameter: price for limit order",
            ));
        }

        // Build the payload
        let mut payload = json!({
            "symbol": symbol,
            "side": side,
            "type": order_type,
            "post_only": post_only.unwrap_or(false),
        });

        if let Some(base_qty) = base_quantity {
            payload["base_quantity"] = json!(base_qty);
        }
        if let Some(quote_qty) = quote_quantity {
            payload["quote_quantity"] = json!(quote_qty);
        }
        if let Some(p) = price {
            payload["price"] = json!(p);
        }
        if let Some(slippage) = max_slippage_basis_point {
            payload["max_slippage_basis_point"] = json!(slippage);
        }

        // Send the request
        let url = format!("{}/build", self.path_url);
        let response = self.api.post(&url, payload).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("build_place_order_transaction"))?)
    }

    /// Submits a place order transaction.
    ///
    /// Submits a signed transaction to place an order on the DeltaDeFi platform.
    /// This is the second step in the two-phase order placement process.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID returned from `build_place_order_transaction`
    /// * `signed_tx` - The signed transaction hex string
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the order details or a `WError` if submission fails.
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

    /// Cancels a single order by order ID.
    ///
    /// This method directly cancels an order without requiring transaction signing.
    /// The cancellation is processed immediately by the DeltaDeFi platform.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The ID of the order to cancel
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the cancelled order ID or a `WError` if cancellation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Cancel an order by its ID
    /// let response = client.order.cancel_order("order-id-123").await?;
    /// println!("Cancelled order: {}", response.order_id);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Order ID is invalid or order doesn't exist
    /// - Order cannot be cancelled (already filled or cancelled)
    /// - Network request fails
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse, WError> {
        let url = format!("{}/{}/cancel", self.path_url, order_id);
        let response = self.api.post(&url, json!({})).await?;
        Ok(serde_json::from_str(&response).map_err(WError::from_err("cancel_order"))?)
    }

    /// Cancels all open orders for a specific trading symbol.
    ///
    /// This method directly cancels all open orders for the specified symbol
    /// without requiring transaction signing. The cancellation is processed
    /// immediately by the DeltaDeFi platform.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol (e.g., "ADAUSDM")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the symbol and list of cancelled order IDs,
    /// or a `WError` if cancellation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Cancel all open orders for ADAUSDM
    /// let response = client.order.cancel_all_orders("ADAUSDM").await?;
    /// println!("Cancelled {} orders for {}", response.order_ids.len(), response.symbol);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Symbol is invalid
    /// - No open orders exist to cancel
    /// - Network request fails
    pub async fn cancel_all_orders(
        &self,
        symbol: &str,
    ) -> Result<CancelAllOrdersResponse, WError> {
        let url = format!("{}/cancel-all", self.path_url);
        let payload = json!({
            "symbol": symbol,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(serde_json::from_str(&response).map_err(WError::from_err("cancel_all_orders"))?)
    }
}
