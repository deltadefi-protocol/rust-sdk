use serde_json::json;
use whisky::WError;

use crate::{
    order::{
        BuildCancelOrderTransactionResponse, BuildPlaceOrderTransactionResponse,
        SubmitCancelOrderTransactionResponse, SubmitPlaceOrderTransactionResponse,
    },
    OrderSide, OrderType,
};

use super::Api;

pub struct Order {
    pub api: Api,
    pub path_url: String,
}

impl Order {
    pub fn new(api: Api) -> Self {
        Order {
            api,
            path_url: "/order".to_string(),
        }
    }

    /// Builds a place order transaction.
    pub async fn build_place_order_transaction(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        limit_slippage: Option<bool>,
        max_slippage_basis_point: Option<u64>,
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
    pub async fn submit_cancel_order_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitCancelOrderTransactionResponse, WError> {
        let url = format!("{}/submit", self.path_url);
        let payload = json!({
            "signed_tx": signed_tx,
        });
        let response = self.api.delete(&url, payload).await?;
        Ok(serde_json::from_str(&response)
            .map_err(WError::from_err("submit_cancel_order_transaction"))?)
    }
}
