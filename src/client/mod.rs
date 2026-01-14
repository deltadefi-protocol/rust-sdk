//! DeltaDeFi Client Module
//!
//! This module provides the core client functionality for interacting with the DeltaDeFi API.
//! It includes the main `DeltaDeFi` client struct and supporting API infrastructure.

mod accounts;
mod market;
mod market_stream;
mod order;
mod stream;

use accounts::Accounts;
use market::Market;
use order::Order;
pub use market_stream::{MarketStream, MarketStreamEvent, MarketStreamHandle};
pub use stream::{AccountStream, ReconnectConfig, StreamError, StreamEvent, StreamHandle};

use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use whisky::{decrypt_with_cipher, WError, Wallet, WalletType};

use crate::{
    order::{CancelAllOrdersResponse, CancelOrderResponse, SubmitPlaceOrderTransactionResponse},
    OrderSide, OrderType,
};

/// Network environment configuration for DeltaDeFi API endpoints.
///
/// Specifies which network environment to connect to, allowing for testing
/// and development on different stages of the DeltaDeFi protocol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    /// Production mainnet environment
    Mainnet,
    /// Staging environment for testing
    Staging,
    /// Custom API endpoint URL
    Custom(String, String),
}

/// The main DeltaDeFi client for interacting with the DeltaDeFi protocol.
///
/// This struct provides access to all DeltaDeFi functionality including:
/// - Account management (deposits, withdrawals, balances)
/// - Market data (prices, historical data)
/// - Order operations (place, cancel, track orders)
/// - Wallet operations (transaction signing)
/// - Real-time streaming (account updates, market data)
///
/// # Examples
///
/// ```rust
/// use deltadefi::{DeltaDeFi, Stage};
///
/// let client = DeltaDeFi::new(
///     "your-api-key".to_string(),
///     Stage::Staging,
///     None
/// )?;
/// ```
pub struct DeltaDeFi {
    /// Account management operations
    pub accounts: Accounts,
    /// Market data operations
    pub market: Market,
    /// Order management operations
    pub order: Order,
    /// Account stream for real-time account updates (balance, orders, points)
    pub stream: AccountStream,
    /// Market stream for real-time market data (depth, price, trades, OHLC)
    pub market_stream: MarketStream,
    /// Optional master wallet for transaction signing
    pub master_wallet: Option<Wallet>,
    /// Optional operation wallet for transaction signing
    pub operation_wallet: Option<Wallet>,
}

impl DeltaDeFi {
    /// Creates a new DeltaDeFi client instance.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your DeltaDeFi API key for authentication
    /// * `network` - The network environment to connect to
    /// * `master_key` - Optional master wallet key for transaction signing
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the initialized client or a `WError` if initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use deltadefi::{DeltaDeFi, Stage};
    ///
    /// // Basic client without wallet
    /// let client = DeltaDeFi::new(
    ///     "your-api-key".to_string(),
    ///     Stage::Staging,
    ///     None
    /// )?;
    ///
    /// // Client with master wallet
    /// use whisky::WalletType;
    /// let client = DeltaDeFi::new(
    ///     "your-api-key".to_string(),
    ///     Stage::Mainnet,
    ///     Some(WalletType::Mnemonic("your seed phrase".to_string()))
    /// )?;
    /// ```
    pub fn new(
        api_key: String,
        network: Stage,
        master_key: Option<WalletType>,
    ) -> Result<Self, WError> {
        let master_wallet = match master_key {
            Some(key) => Some(Wallet::new(key).map_err(WError::from_err("DeltaDeFi - new"))?),
            None => None,
        };

        let api = Api::new(api_key, network);

        Ok(DeltaDeFi {
            accounts: Accounts::new(api.clone()),
            market: Market::new(api.clone()),
            order: Order::new(api.clone()),
            stream: AccountStream::new(api.ws_url.clone(), api.api_key.clone()),
            market_stream: MarketStream::new(api.ws_url.clone()),
            master_wallet,
            operation_wallet: None,
        })
    }

    /// Loads the operation key required for transaction signing.
    ///
    /// This method fetches the encrypted operation key from the DeltaDeFi API,
    /// decrypts it using the provided password, and stores it in the client
    /// for subsequent transaction signing operations.
    ///
    /// # Arguments
    ///
    /// * `password` - The password used to decrypt the operation key
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. On success, the operation
    /// wallet is available for signing transactions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut client = DeltaDeFi::new(api_key, Stage::Staging, None)?;
    /// client.load_operation_key("your-password").await?;
    ///
    /// // Now you can sign transactions
    /// let signed_tx = client.sign_tx_by_operation_key(&tx_hex)?;
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The API request fails
    /// - The password is incorrect
    /// - The operation key cannot be decrypted
    pub async fn load_operation_key(&mut self, password: &str) -> Result<(), WError> {
        let res = self
            .accounts
            .get_operation_key()
            .await
            .map_err(WError::from_err("DeltaDeFi - load_operation_key"))?;
        let operation_key = decrypt_with_cipher(&res.encrypted_operation_key, password).map_err(
            WError::from_err("DeltaDeFi - load_operation_key - decrypt_with_cipher"),
        )?;
        let operation_wallet = Wallet::new_root_key(&operation_key).map_err(WError::from_err(
            "DeltaDeFi - load_operation_key - create operation wallet",
        ))?;
        self.operation_wallet = Some(operation_wallet);
        Ok(())
    }

    /// Signs a transaction using the master wallet key.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction hex string to sign
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the signed transaction hex string or a `WError` if signing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let signed_tx = client.sign_tx_by_master_key(&tx_hex)?;
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - No master wallet is loaded
    /// - The transaction signing fails
    pub fn sign_tx_by_master_key(&self, tx: &str) -> Result<String, WError> {
        if let Some(wallet) = &self.master_wallet {
            wallet.sign_tx(tx)
        } else {
            Err(WError::new("DeltaDeFi", "No wallet found"))
        }
    }

    /// Signs a transaction using the operation wallet key.
    ///
    /// This is the preferred method for signing most transactions as it uses
    /// the operation key which is specifically designed for trading operations.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction hex string to sign
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the signed transaction hex string or a `WError` if signing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Make sure operation key is loaded first
    /// client.load_operation_key("password").await?;
    /// let signed_tx = client.sign_tx_by_operation_key(&tx_hex)?;
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - No operation wallet is loaded (call `load_operation_key` first)
    /// - The transaction signing fails
    pub fn sign_tx_by_operation_key(&self, tx: &str) -> Result<String, WError> {
        if let Some(wallet) = &self.operation_wallet {
            wallet.sign_tx(tx)
        } else {
            Err(WError::new("DeltaDeFi", "No wallet found"))
        }
    }

    /// Convenience method to place an order with automatic transaction building and signing.
    ///
    /// This method combines building the order transaction, signing it with the operation key,
    /// and submitting it to the DeltaDeFi protocol in a single call.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol (e.g., "ADAUSDM")
    /// * `side` - Order side: `OrderSide::Buy` or `OrderSide::Sell`
    /// * `order_type` - Order type: `OrderType::Market` or `OrderType::Limit`
    /// * `base_quantity` - The base asset quantity (mutually exclusive with quote_quantity)
    /// * `quote_quantity` - The quote asset quantity (mutually exclusive with base_quantity)
    /// * `price` - Required for limit orders, ignored for market orders
    /// * `max_slippage_basis_point` - Maximum slippage in basis points (100 = 1%)
    /// * `post_only` - If true, the order will only be posted to the order book
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the order submission response with order details.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Place a limit buy order with base quantity
    /// let response = client.post_order(
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
    /// // Place a market sell order with slippage protection
    /// let response = client.post_order(
    ///     "ADAUSDM",
    ///     OrderSide::Sell,
    ///     OrderType::Market,
    ///     Some("50.0".to_string()),   // base quantity
    ///     None,
    ///     None,                        // No price for market orders
    ///     Some("100".to_string()),     // Max 1% slippage
    ///     None,
    /// ).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - No operation wallet is loaded (call `load_operation_key` first)
    /// - Order parameters are invalid
    /// - Network request fails
    /// - Transaction signing fails
    pub async fn post_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        base_quantity: Option<String>,
        quote_quantity: Option<String>,
        price: Option<String>,
        max_slippage_basis_point: Option<String>,
        post_only: Option<bool>,
    ) -> Result<SubmitPlaceOrderTransactionResponse, WError> {
        let build_res = self
            .order
            .build_place_order_transaction(
                symbol,
                side,
                order_type,
                base_quantity,
                quote_quantity,
                price,
                max_slippage_basis_point,
                post_only,
            )
            .await?;
        let signed_tx = self.sign_tx_by_operation_key(&build_res.tx_hex)?;
        self.order
            .submit_place_order_transaction(&build_res.order_id, &signed_tx)
            .await
    }

    /// Convenience method to cancel an existing order.
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
    /// let response = client.cancel_order("order-id-123").await?;
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
        self.order.cancel_order(order_id).await
    }

    /// Convenience method to cancel all open orders for a specific trading symbol.
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
    /// let response = client.cancel_all_orders("ADAUSDM").await?;
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
        self.order.cancel_all_orders(symbol).await
    }

    /// Subscribe to the account stream for real-time updates.
    ///
    /// This method establishes a WebSocket connection to receive real-time
    /// notifications about balance changes, order updates, and DLTA points.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Optional size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `StreamHandle` - Handle to control the stream connection
    /// - `mpsc::Receiver<StreamMessage>` - Receiver for stream messages
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use deltadefi::{DeltaDeFi, Stage, StreamMessage};
    ///
    /// let client = DeltaDeFi::new("api-key".to_string(), Stage::Staging, None)?;
    ///
    /// let (mut handle, mut receiver) = client.subscribe_account_stream(None).await?;
    ///
    /// // Process messages
    /// while let Some(message) = receiver.recv().await {
    ///     match message {
    ///         StreamMessage::Balance(msg) => {
    ///             for balance in &msg.balance {
    ///                 println!("{}: free={}, locked={}", balance.asset, balance.free, balance.locked);
    ///             }
    ///         }
    ///         StreamMessage::OrderInfo(msg) => {
    ///             println!("Order {} is now {:?}", msg.order.id, msg.order.status);
    ///         }
    ///         StreamMessage::DltaPoints(msg) => {
    ///             println!("Earned {} points! Total: {}", msg.dlta_points.delta, msg.dlta_points.new_total);
    ///         }
    ///         StreamMessage::Unknown(raw) => {
    ///             eprintln!("Unknown message: {}", raw);
    ///         }
    ///     }
    /// }
    ///
    /// // Close when done
    /// handle.close().await;
    /// ```
    pub async fn subscribe_account_stream(
        &self,
        buffer_size: Option<usize>,
    ) -> Result<(StreamHandle, tokio::sync::mpsc::Receiver<crate::responses::stream::StreamMessage>), WError> {
        self.stream.subscribe(buffer_size).await
    }
}

/// Internal API client for handling HTTP requests to DeltaDeFi endpoints.
///
/// This struct manages the HTTP client, authentication, and request routing
/// for all API operations. It's used internally by the various module clients.
#[derive(Clone)]
pub struct Api {
    /// Base URL for the DeltaDeFi API
    pub base_url: String,
    /// Websocket URL for the DeltaDeFi stream
    pub ws_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Network environment configuration
    pub network: Stage,
    /// HTTP client for making requests
    pub http_client: reqwest::Client,
}

impl Api {
    pub fn new(api_key: String, network: Stage) -> Self {
        let (base_url, ws_url) = match &network {
            Stage::Mainnet => (
                "https://api.deltadefi.io/".to_string(),
                "wss://stream.deltadefi.io".to_string(),
            ),
            Stage::Staging => (
                "https://api-staging.deltadefi.io".to_string(),
                "wss://stream-staging.deltadefi.io".to_string(),
            ),
            Stage::Custom(url, ws_url) => (url.to_string(), ws_url.to_string()),
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Api {
            api_key,
            network,
            base_url,
            ws_url,
            http_client,
        }
    }

    async fn send_request(
        &self,
        req: RequestBuilder,
        response_body: &mut String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let req = req
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.api_key)
            .build()?;

        // Capture request details for error logging
        let method = req.method().to_string();
        let url = req.url().to_string();

        let response = self.http_client.execute(req).await?;
        let status = response.status();

        if status.is_success() {
            *response_body = response.text().await?;
            Ok(())
        } else {
            // Get response body for error details
            let error_body = response.text().await.unwrap_or_else(|_| "<no body>".to_string());
            Err(format!(
                "HTTP {} {} -> {} | Response: {}",
                method, url, status, error_body
            ).into())
        }
    }

    pub async fn get(&self, url: &str) -> Result<String, WError> {
        let req = self.http_client.get(format!("{}{}", &self.base_url, url));
        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(WError::from_err("DeltaDeFi - get - send_request"))?;
        Ok(response_body)
    }

    pub async fn get_with_params<T: Serialize>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<String, WError> {
        let req = self
            .http_client
            .get(format!("{}{}", &self.base_url, url))
            .query(params);

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(WError::from_err(
                "DeltaDeFi - get_with_params - send_request",
            ))?;
        Ok(response_body)
    }

    pub async fn post<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - post - json_body"))?;

        let req = self
            .http_client
            .post(format!("{}{}", &self.base_url, url))
            .header("Content-Type", "application/json")
            .body(json_body.clone());

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(|e| {
                // Include request body in error for debugging
                WError::new(
                    "DeltaDeFi - post - send_request",
                    &format!("{} | Request body: {}", e, json_body),
                )
            })?;
        Ok(response_body)
    }

    pub async fn delete<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - delete - json_body"))?;

        let req = self
            .http_client
            .delete(format!("{}{}", &self.base_url, url))
            .header("Content-Type", "application/json")
            .body(json_body.clone());

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(|e| {
                WError::new(
                    "DeltaDeFi - delete - send_request",
                    &format!("{} | Request body: {}", e, json_body),
                )
            })?;
        Ok(response_body)
    }

    pub async fn patch<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - patch - json_body"))?;

        let req = self
            .http_client
            .patch(format!("{}{}", &self.base_url, url))
            .header("Content-Type", "application/json")
            .body(json_body.clone());

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(|e| {
                WError::new(
                    "DeltaDeFi - patch - send_request",
                    &format!("{} | Request body: {}", e, json_body),
                )
            })?;
        Ok(response_body)
    }
}
