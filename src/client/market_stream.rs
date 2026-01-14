//! Market Stream Module
//!
//! This module provides WebSocket streaming functionality for real-time market data.
//! It enables subscribing to market depth, price changes, recent trades, and OHLC data.
//!
//! ## Automatic Reconnection
//!
//! All market streams support automatic reconnection with exponential backoff.
//! Use `subscribe_*_with_reconnect` methods for resilient connections.
//!
//! ## Example with Reconnection
//!
//! ```rust,ignore
//! use deltadefi::{DeltaDeFi, Stage, ReconnectConfig};
//!
//! let client = DeltaDeFi::new("api-key".to_string(), Stage::Staging, None)?;
//!
//! let config = ReconnectConfig::default().with_max_retries(10);
//! let (mut handle, mut receiver) = client.market_stream
//!     .subscribe_depth_with_reconnect("ADAUSDM", None, Some(config)).await?;
//!
//! while let Some(event) = receiver.recv().await {
//!     match event {
//!         MarketStreamEvent::Message(msg) => println!("Market data: {:?}", msg),
//!         MarketStreamEvent::Connected => println!("Connected to market stream!"),
//!         MarketStreamEvent::Reconnecting { attempt, delay_ms } => {
//!             println!("Reconnecting attempt {} in {}ms", attempt, delay_ms);
//!         }
//!         MarketStreamEvent::Disconnected { reason } => println!("Disconnected: {}", reason),
//!         MarketStreamEvent::MaxRetriesExceeded => break,
//!     }
//! }
//! ```

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use whisky::WError;

use crate::client::stream::ReconnectConfig;
use crate::responses::stream::{
    MarketDepthMessage, MarketPriceMessage, MarketStreamMessage, OhlcMessage, Trade,
};

/// Events emitted by market streams, including connection lifecycle events.
#[derive(Debug, Clone)]
pub enum MarketStreamEvent {
    /// A message was received from the stream.
    Message(MarketStreamMessage),
    /// Successfully connected (or reconnected) to the stream.
    Connected,
    /// Connection was lost, attempting to reconnect.
    Reconnecting {
        /// Current attempt number (1-indexed)
        attempt: u32,
        /// Delay before this attempt (milliseconds)
        delay_ms: u64,
    },
    /// Disconnected from the stream.
    Disconnected {
        /// Reason for disconnection
        reason: String,
    },
    /// Maximum reconnection attempts exceeded.
    MaxRetriesExceeded,
}

/// Handle for controlling an active market stream connection.
///
/// This handle can be used to close the stream connection gracefully.
/// The connection will also be closed when this handle is dropped.
#[derive(Debug)]
pub struct MarketStreamHandle {
    close_tx: Option<mpsc::Sender<()>>,
}

impl MarketStreamHandle {
    /// Close the stream connection gracefully.
    pub async fn close(&mut self) {
        if let Some(tx) = self.close_tx.take() {
            let _ = tx.send(()).await;
        }
    }

    /// Check if the stream is still active.
    pub fn is_active(&self) -> bool {
        self.close_tx.is_some()
    }
}

impl Drop for MarketStreamHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.close_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

/// Internal result type for connection attempts
enum ConnectionResult {
    UserClosed,
    /// Connection error. The boolean indicates whether connection was ever established.
    Error(String, bool),
    ReceiverDropped,
}

/// Market data stream client for receiving real-time market updates.
///
/// This client manages WebSocket connections to the DeltaDeFi market streaming API
/// and provides parsed messages through async channels.
///
/// # Available Streams
///
/// - **Depth Stream**: Order book depth updates
/// - **Price Stream**: Market price changes
/// - **Recent Trades Stream**: Recent trade executions
/// - **OHLC/Graph Stream**: Candlestick chart data
///
/// # Example
///
/// ```rust,ignore
/// use deltadefi::{DeltaDeFi, Stage};
///
/// let client = DeltaDeFi::new("api-key".to_string(), Stage::Staging, None)?;
///
/// // Subscribe to market depth for ADAUSDM
/// let (mut handle, mut receiver) = client.market_stream
///     .subscribe_depth("ADAUSDM", None).await?;
///
/// while let Some(message) = receiver.recv().await {
///     if let MarketStreamMessage::Depth(depth) = message {
///         println!("Bids: {:?}, Asks: {:?}", depth.bids.len(), depth.asks.len());
///     }
/// }
///
/// handle.close().await;
/// ```
pub struct MarketStream {
    ws_url: String,
}

impl MarketStream {
    /// Create a new MarketStream instance.
    pub fn new(ws_url: String) -> Self {
        MarketStream { ws_url }
    }

    // ========================================================================
    // Basic subscription methods (no reconnection)
    // ========================================================================

    /// Subscribe to the market depth stream for a symbol.
    ///
    /// Receives order book depth updates showing bids and asks.
    /// This is the simple API without automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing the stream handle and message receiver.
    pub async fn subscribe_depth(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamMessage>), WError> {
        let ws_endpoint = format!("{}/market/ws/depth/{}", self.ws_url, symbol);
        self.connect_and_stream(ws_endpoint, buffer_size, |json| {
            match serde_json::from_str::<MarketDepthMessage>(json) {
                Ok(msg) => MarketStreamMessage::Depth(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to the market price stream for a symbol.
    ///
    /// Receives price updates when trades occur.
    /// This is the simple API without automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing the stream handle and message receiver.
    pub async fn subscribe_price(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamMessage>), WError> {
        let ws_endpoint = format!("{}/market/ws/market-price/{}", self.ws_url, symbol);
        self.connect_and_stream(ws_endpoint, buffer_size, |json| {
            match serde_json::from_str::<MarketPriceMessage>(json) {
                Ok(msg) => MarketStreamMessage::Price(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to the recent trades stream for a symbol.
    ///
    /// Receives notifications about recent trade executions.
    /// This is the simple API without automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing the stream handle and message receiver.
    pub async fn subscribe_recent_trades(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamMessage>), WError> {
        let ws_endpoint = format!("{}/market/ws/recent-trade/{}", self.ws_url, symbol);
        self.connect_and_stream(ws_endpoint, buffer_size, |json| {
            match serde_json::from_str::<Vec<Trade>>(json) {
                Ok(trades) => MarketStreamMessage::RecentTrades(trades),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to the OHLC/graph stream for a symbol.
    ///
    /// Receives candlestick chart data updates.
    /// This is the simple API without automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `interval` - Time interval (e.g., "5m", "15m", "30m", "1h", "1d")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing the stream handle and message receiver.
    pub async fn subscribe_ohlc(
        &self,
        symbol: &str,
        interval: &str,
        buffer_size: Option<usize>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamMessage>), WError> {
        let ws_endpoint = format!("{}/market/ws/graph/{}/{}", self.ws_url, symbol, interval);
        self.connect_and_stream(ws_endpoint, buffer_size, |json| {
            match serde_json::from_str::<OhlcMessage>(json) {
                Ok(msg) => MarketStreamMessage::Ohlc(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Internal method to connect and stream messages (no reconnection).
    async fn connect_and_stream<F>(
        &self,
        ws_endpoint: String,
        buffer_size: Option<usize>,
        parser: F,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamMessage>), WError>
    where
        F: Fn(&str) -> MarketStreamMessage + Send + 'static,
    {
        let buffer = buffer_size.unwrap_or(100);
        let (message_tx, message_rx) = mpsc::channel::<MarketStreamMessage>(buffer);
        let (close_tx, mut close_rx) = mpsc::channel::<()>(1);

        // Connect to WebSocket with timeout
        let connect_timeout = Duration::from_secs(30);
        let (ws_stream, _response) = timeout(connect_timeout, connect_async(&ws_endpoint))
            .await
            .map_err(|_| WError::new("MarketStream", "Connection timeout"))?
            .map_err(|e| WError::new("MarketStream", &format!("Connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // Spawn background task to handle WebSocket messages
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Check for close signal
                    _ = close_rx.recv() => {
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                    // Handle incoming messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                let stream_msg = parser(&text);
                                if message_tx.send(stream_msg).await.is_err() {
                                    let _ = write.send(Message::Close(None)).await;
                                    break;
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                if write.send(Message::Pong(data)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                break;
                            }
                            Some(Err(_)) => {
                                break;
                            }
                            None => {
                                break;
                            }
                            _ => {
                                // Ignore other message types
                            }
                        }
                    }
                }
            }
        });

        let handle = MarketStreamHandle {
            close_tx: Some(close_tx),
        };

        Ok((handle, message_rx))
    }

    // ========================================================================
    // Subscription methods with automatic reconnection
    // ========================================================================

    /// Subscribe to depth stream with automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    /// * `reconnect_config` - Configuration for reconnection behavior
    pub async fn subscribe_depth_with_reconnect(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamEvent>), WError> {
        let ws_endpoint = format!("{}/market/ws/depth/{}", self.ws_url, symbol);
        self.connect_and_stream_with_reconnect(ws_endpoint, buffer_size, reconnect_config, |json| {
            match serde_json::from_str::<MarketDepthMessage>(json) {
                Ok(msg) => MarketStreamMessage::Depth(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to price stream with automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    /// * `reconnect_config` - Configuration for reconnection behavior
    pub async fn subscribe_price_with_reconnect(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamEvent>), WError> {
        let ws_endpoint = format!("{}/market/ws/market-price/{}", self.ws_url, symbol);
        self.connect_and_stream_with_reconnect(ws_endpoint, buffer_size, reconnect_config, |json| {
            match serde_json::from_str::<MarketPriceMessage>(json) {
                Ok(msg) => MarketStreamMessage::Price(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to recent trades stream with automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    /// * `reconnect_config` - Configuration for reconnection behavior
    pub async fn subscribe_recent_trades_with_reconnect(
        &self,
        symbol: &str,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamEvent>), WError> {
        let ws_endpoint = format!("{}/market/ws/recent-trade/{}", self.ws_url, symbol);
        self.connect_and_stream_with_reconnect(ws_endpoint, buffer_size, reconnect_config, |json| {
            match serde_json::from_str::<Vec<Trade>>(json) {
                Ok(trades) => MarketStreamMessage::RecentTrades(trades),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Subscribe to OHLC stream with automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "ADAUSDM")
    /// * `interval` - Time interval (e.g., "5m", "15m", "30m", "1h", "1d")
    /// * `buffer_size` - Size of the message buffer (default: 100)
    /// * `reconnect_config` - Configuration for reconnection behavior
    pub async fn subscribe_ohlc_with_reconnect(
        &self,
        symbol: &str,
        interval: &str,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamEvent>), WError> {
        let ws_endpoint = format!("{}/market/ws/graph/{}/{}", self.ws_url, symbol, interval);
        self.connect_and_stream_with_reconnect(ws_endpoint, buffer_size, reconnect_config, |json| {
            match serde_json::from_str::<OhlcMessage>(json) {
                Ok(msg) => MarketStreamMessage::Ohlc(msg),
                Err(_) => MarketStreamMessage::Unknown(json.to_string()),
            }
        })
        .await
    }

    /// Internal method to connect and stream with automatic reconnection.
    async fn connect_and_stream_with_reconnect<F>(
        &self,
        ws_endpoint: String,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
        parser: F,
    ) -> Result<(MarketStreamHandle, mpsc::Receiver<MarketStreamEvent>), WError>
    where
        F: Fn(&str) -> MarketStreamMessage + Send + Sync + 'static,
    {
        let buffer = buffer_size.unwrap_or(100);
        let config = reconnect_config.unwrap_or_default();
        let (event_tx, event_rx) = mpsc::channel::<MarketStreamEvent>(buffer);
        let (close_tx, close_rx) = mpsc::channel::<()>(1);

        // Spawn the reconnecting stream task
        tokio::spawn(Self::run_reconnecting_stream(
            ws_endpoint,
            config,
            event_tx,
            close_rx,
            parser,
        ));

        let handle = MarketStreamHandle {
            close_tx: Some(close_tx),
        };

        Ok((handle, event_rx))
    }

    /// Internal: Run a single WebSocket connection, returning why it ended.
    async fn run_single_connection<F>(
        ws_endpoint: &str,
        connect_timeout: Duration,
        event_tx: &mpsc::Sender<MarketStreamEvent>,
        close_rx: &mut mpsc::Receiver<()>,
        parser: &F,
    ) -> ConnectionResult
    where
        F: Fn(&str) -> MarketStreamMessage,
    {
        // Attempt to connect with timeout
        let connect_result = timeout(connect_timeout, connect_async(ws_endpoint)).await;

        let ws_stream = match connect_result {
            Ok(Ok((stream, _response))) => stream,
            Ok(Err(e)) => {
                return ConnectionResult::Error(format!("Connection failed: {}", e), false);
            }
            Err(_) => {
                return ConnectionResult::Error("Connection timeout".to_string(), false);
            }
        };

        // Notify connected
        if event_tx.send(MarketStreamEvent::Connected).await.is_err() {
            return ConnectionResult::ReceiverDropped;
        }

        let (mut write, mut read) = ws_stream.split();

        // Message loop
        loop {
            tokio::select! {
                // Check for close signal
                _ = close_rx.recv() => {
                    let _ = write.send(Message::Close(None)).await;
                    return ConnectionResult::UserClosed;
                }
                // Handle incoming messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let stream_msg = parser(&text);
                            let event = MarketStreamEvent::Message(stream_msg);
                            if event_tx.send(event).await.is_err() {
                                let _ = write.send(Message::Close(None)).await;
                                return ConnectionResult::ReceiverDropped;
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if write.send(Message::Pong(data)).await.is_err() {
                                return ConnectionResult::Error("Failed to send pong".to_string(), true);
                            }
                        }
                        Some(Ok(Message::Close(frame))) => {
                            let reason = frame
                                .map(|f| f.reason.to_string())
                                .unwrap_or_else(|| "Server closed connection".to_string());
                            return ConnectionResult::Error(reason, true);
                        }
                        Some(Err(e)) => {
                            return ConnectionResult::Error(format!("WebSocket error: {}", e), true);
                        }
                        None => {
                            return ConnectionResult::Error("Stream ended unexpectedly".to_string(), true);
                        }
                        _ => {
                            // Ignore other message types
                        }
                    }
                }
            }
        }
    }

    /// Internal: Run the reconnecting stream loop.
    async fn run_reconnecting_stream<F>(
        ws_endpoint: String,
        config: ReconnectConfig,
        event_tx: mpsc::Sender<MarketStreamEvent>,
        mut close_rx: mpsc::Receiver<()>,
        parser: F,
    ) where
        F: Fn(&str) -> MarketStreamMessage + Send + Sync + 'static,
    {
        let connect_timeout = Duration::from_millis(config.connect_timeout_ms);
        let mut attempt: u32 = 0;

        loop {
            // Check for close signal before attempting connection
            if close_rx.try_recv().is_ok() {
                break;
            }

            let result = Self::run_single_connection(
                &ws_endpoint,
                connect_timeout,
                &event_tx,
                &mut close_rx,
                &parser,
            )
            .await;

            match result {
                ConnectionResult::UserClosed => {
                    break;
                }
                ConnectionResult::ReceiverDropped => {
                    break;
                }
                ConnectionResult::Error(reason, was_connected) => {
                    // Reset attempt counter if we had a successful connection
                    // This ensures fresh backoff after a working session ends
                    if was_connected {
                        attempt = 0;
                    }

                    // Send disconnected event
                    let _ = event_tx
                        .send(MarketStreamEvent::Disconnected {
                            reason: reason.clone(),
                        })
                        .await;

                    // Check if we should retry
                    if !config.should_retry(attempt) {
                        let _ = event_tx.send(MarketStreamEvent::MaxRetriesExceeded).await;
                        break;
                    }

                    // Calculate delay and notify
                    let delay_ms = config.delay_for_attempt(attempt);
                    attempt += 1;

                    let _ = event_tx
                        .send(MarketStreamEvent::Reconnecting { attempt, delay_ms })
                        .await;

                    // Wait for delay, but allow early exit on close signal
                    tokio::select! {
                        _ = close_rx.recv() => {
                            break;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(delay_ms)) => {
                            // Continue to retry
                        }
                    }
                }
            }
        }
    }

    /// Subscribe to depth stream with a callback function.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `callback` - Async function called for each message. Return `false` to stop.
    pub async fn subscribe_depth_with_callback<F, Fut>(
        &self,
        symbol: &str,
        mut callback: F,
    ) -> Result<(), WError>
    where
        F: FnMut(MarketStreamMessage) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe_depth(symbol, None).await?;

        while let Some(message) = receiver.recv().await {
            if !callback(message).await {
                break;
            }
        }

        handle.close().await;
        Ok(())
    }

    /// Subscribe to price stream with a callback function.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `callback` - Async function called for each message. Return `false` to stop.
    pub async fn subscribe_price_with_callback<F, Fut>(
        &self,
        symbol: &str,
        mut callback: F,
    ) -> Result<(), WError>
    where
        F: FnMut(MarketStreamMessage) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe_price(symbol, None).await?;

        while let Some(message) = receiver.recv().await {
            if !callback(message).await {
                break;
            }
        }

        handle.close().await;
        Ok(())
    }

    /// Subscribe to recent trades stream with a callback function.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `callback` - Async function called for each message. Return `false` to stop.
    pub async fn subscribe_recent_trades_with_callback<F, Fut>(
        &self,
        symbol: &str,
        mut callback: F,
    ) -> Result<(), WError>
    where
        F: FnMut(MarketStreamMessage) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe_recent_trades(symbol, None).await?;

        while let Some(message) = receiver.recv().await {
            if !callback(message).await {
                break;
            }
        }

        handle.close().await;
        Ok(())
    }

    /// Subscribe to OHLC stream with a callback function.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Time interval (e.g., "5m", "1h")
    /// * `callback` - Async function called for each message. Return `false` to stop.
    pub async fn subscribe_ohlc_with_callback<F, Fut>(
        &self,
        symbol: &str,
        interval: &str,
        mut callback: F,
    ) -> Result<(), WError>
    where
        F: FnMut(MarketStreamMessage) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe_ohlc(symbol, interval, None).await?;

        while let Some(message) = receiver.recv().await {
            if !callback(message).await {
                break;
            }
        }

        handle.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_depth_parsing() {
        let json = r#"{
            "timestamp": 1704067200000,
            "bids": [
                {"price": 0.45, "quantity": 1000.0},
                {"price": 0.44, "quantity": 2000.0}
            ],
            "asks": [
                {"price": 0.46, "quantity": 500.0},
                {"price": 0.47, "quantity": 1500.0}
            ]
        }"#;

        let msg: MarketDepthMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.timestamp, 1704067200000);
        assert_eq!(msg.bids.len(), 2);
        assert_eq!(msg.asks.len(), 2);
        assert_eq!(msg.bids[0].price, 0.45);
        assert_eq!(msg.bids[0].quantity, 1000.0);
    }

    #[test]
    fn test_market_price_parsing() {
        let json = r#"{"price": 0.456789}"#;

        let msg: MarketPriceMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.price, 0.456789);
    }

    #[test]
    fn test_recent_trades_parsing() {
        let json = r#"[
            {
                "order_id": "order-123",
                "timestamp": "2024-01-01T00:00:00Z",
                "symbol": "ADAUSDM",
                "price": 0.45,
                "amount": 100.0,
                "side": "buy"
            }
        ]"#;

        let trades: Vec<Trade> = serde_json::from_str(json).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].order_id, "order-123");
        assert_eq!(trades[0].symbol, "ADAUSDM");
        assert_eq!(trades[0].side, "buy");
    }

    #[test]
    fn test_ohlc_parsing() {
        let json = r#"{
            "t": 1704067200,
            "s": "ADAUSDM",
            "o": 0.45,
            "h": 0.48,
            "l": 0.44,
            "c": 0.47,
            "v": 10000.5
        }"#;

        let msg: OhlcMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.timestamp, 1704067200);
        assert_eq!(msg.symbol, "ADAUSDM");
        assert_eq!(msg.open, 0.45);
        assert_eq!(msg.high, 0.48);
        assert_eq!(msg.low, 0.44);
        assert_eq!(msg.close, 0.47);
        assert_eq!(msg.volume, 10000.5);
    }

    #[test]
    fn test_market_stream_message_helpers() {
        let depth = MarketStreamMessage::Depth(MarketDepthMessage {
            timestamp: 0,
            bids: vec![],
            asks: vec![],
        });
        assert!(depth.is_depth());
        assert!(!depth.is_price());

        let price = MarketStreamMessage::Price(MarketPriceMessage { price: 0.5 });
        assert!(price.is_price());
        assert!(!price.is_depth());

        let trades = MarketStreamMessage::RecentTrades(vec![]);
        assert!(trades.is_recent_trades());

        let ohlc = MarketStreamMessage::Ohlc(OhlcMessage {
            timestamp: 0,
            symbol: "TEST".to_string(),
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
        });
        assert!(ohlc.is_ohlc());

        let unknown = MarketStreamMessage::Unknown("test".to_string());
        assert!(unknown.is_unknown());
    }
}
