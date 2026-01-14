//! Account Stream Module
//!
//! This module provides WebSocket streaming functionality for real-time account updates.
//! It enables subscribing to balance changes, order updates, and DLTA points notifications.
//!
//! ## Automatic Reconnection
//!
//! The stream supports automatic reconnection with exponential backoff. When the connection
//! drops unexpectedly, the stream will automatically attempt to reconnect using the configured
//! retry strategy.
//!
//! ## Example with Reconnection
//!
//! ```rust,ignore
//! use deltadefi::{DeltaDeFi, Stage, ReconnectConfig};
//!
//! let client = DeltaDeFi::new("api-key".to_string(), Stage::Staging, None)?;
//!
//! // Subscribe with custom reconnection settings
//! let config = ReconnectConfig::default()
//!     .with_max_retries(10)
//!     .with_initial_delay_ms(1000);
//!
//! let (mut handle, mut receiver) = client.stream
//!     .subscribe_with_reconnect(None, Some(config)).await?;
//!
//! while let Some(event) = receiver.recv().await {
//!     match event {
//!         StreamEvent::Message(msg) => println!("Message: {:?}", msg),
//!         StreamEvent::Connected => println!("Connected!"),
//!         StreamEvent::Reconnecting { attempt, delay_ms } => {
//!             println!("Reconnecting (attempt {}, waiting {}ms)...", attempt, delay_ms);
//!         }
//!         StreamEvent::Disconnected { reason } => {
//!             println!("Disconnected: {}", reason);
//!         }
//!         StreamEvent::MaxRetriesExceeded => {
//!             println!("Max retries exceeded, giving up");
//!             break;
//!         }
//!     }
//! }
//! ```

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use whisky::WError;

use crate::responses::stream::StreamMessage;

/// Configuration for automatic reconnection behavior.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts (0 = no reconnection, None = infinite)
    pub max_retries: Option<u32>,
    /// Initial delay before first reconnection attempt (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between reconnection attempts (milliseconds)
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (e.g., 2.0 = double delay each attempt)
    pub backoff_multiplier: f64,
    /// Connection timeout (milliseconds)
    pub connect_timeout_ms: u64,
    /// Jitter factor for randomizing delays (0.25 = ±25% randomization)
    /// Prevents thundering herd when multiple clients reconnect simultaneously
    pub jitter_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        ReconnectConfig {
            max_retries: Some(10),
            initial_delay_ms: 1000,  // 1 second
            max_delay_ms: 60000,     // 60 seconds
            backoff_multiplier: 2.0,
            connect_timeout_ms: 30000, // 30 seconds
            jitter_factor: 0.25,     // ±25% randomization
        }
    }
}

impl ReconnectConfig {
    /// Create a new ReconnectConfig with no reconnection (single connection attempt).
    pub fn no_reconnect() -> Self {
        ReconnectConfig {
            max_retries: Some(0),
            ..Default::default()
        }
    }

    /// Create a config that retries indefinitely.
    pub fn infinite() -> Self {
        ReconnectConfig {
            max_retries: None,
            ..Default::default()
        }
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the initial delay before reconnection.
    pub fn with_initial_delay_ms(mut self, delay: u64) -> Self {
        self.initial_delay_ms = delay;
        self
    }

    /// Set the maximum delay between retries.
    pub fn with_max_delay_ms(mut self, delay: u64) -> Self {
        self.max_delay_ms = delay;
        self
    }

    /// Set the backoff multiplier.
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set the connection timeout.
    pub fn with_connect_timeout_ms(mut self, timeout: u64) -> Self {
        self.connect_timeout_ms = timeout;
        self
    }

    /// Set the jitter factor (0.0 to 1.0, where 0.25 = ±25% randomization).
    pub fn with_jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given attempt number (0-indexed) with jitter.
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        use rand::Rng;

        let base_delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped = base_delay.min(self.max_delay_ms as f64);

        // Apply jitter: delay * (1 ± jitter_factor)
        if self.jitter_factor > 0.0 {
            let jitter_range = capped * self.jitter_factor;
            let jitter = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
            ((capped + jitter) as u64).max(1) // Ensure at least 1ms
        } else {
            capped as u64
        }
    }

    /// Calculate base delay without jitter (for testing).
    pub fn base_delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        (delay as u64).min(self.max_delay_ms)
    }

    /// Check if should retry for given attempt number.
    pub fn should_retry(&self, attempt: u32) -> bool {
        match self.max_retries {
            None => true,
            Some(max) => attempt < max,
        }
    }
}

/// Events emitted by the stream, including connection lifecycle events.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A message was received from the stream.
    Message(StreamMessage),
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

/// Errors that can occur during stream operations.
#[derive(Debug)]
pub enum StreamError {
    /// WebSocket connection failed
    ConnectionFailed(String),
    /// WebSocket connection was closed
    ConnectionClosed,
    /// Failed to send message through channel
    ChannelSendError,
    /// Stream was already closed
    StreamClosed,
    /// Connection timeout exceeded
    ConnectionTimeout,
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            StreamError::ConnectionClosed => write!(f, "Connection closed"),
            StreamError::ChannelSendError => write!(f, "Failed to send message through channel"),
            StreamError::StreamClosed => write!(f, "Stream was already closed"),
            StreamError::ConnectionTimeout => write!(f, "Connection timeout exceeded"),
        }
    }
}

impl std::error::Error for StreamError {}

impl From<StreamError> for WError {
    fn from(err: StreamError) -> Self {
        WError::new("StreamError", &err.to_string())
    }
}

/// Handle for controlling an active stream connection.
///
/// This handle can be used to close the stream connection gracefully.
/// The connection will also be closed when this handle is dropped.
#[derive(Debug)]
pub struct StreamHandle {
    close_tx: Option<mpsc::Sender<()>>,
}

impl StreamHandle {
    /// Close the stream connection gracefully.
    ///
    /// This will signal the background task to close the WebSocket connection.
    /// After calling this method, no more messages will be received.
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

impl Drop for StreamHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.close_tx.take() {
            // Try to send close signal, ignore if channel is already closed
            let _ = tx.try_send(());
        }
    }
}

/// Internal result type for connection attempts
enum ConnectionResult {
    /// User requested close
    UserClosed,
    /// Connection error (should reconnect)
    /// The boolean indicates whether the connection was ever successfully established
    Error(String, bool),
    /// Receiver dropped (should stop)
    ReceiverDropped,
}

/// Account stream client for receiving real-time updates.
///
/// This client manages WebSocket connections to the DeltaDeFi streaming API
/// and provides parsed messages through an async channel.
///
/// # Example
///
/// ```rust,ignore
/// use deltadefi::{DeltaDeFi, Stage};
///
/// let client = DeltaDeFi::new("api-key".to_string(), Stage::Staging, None)?;
///
/// // Subscribe to account stream
/// let (mut handle, mut receiver) = client.subscribe_account_stream().await?;
///
/// // Process messages in a loop
/// while let Some(message) = receiver.recv().await {
///     match message {
///         StreamMessage::Balance(msg) => {
///             println!("Balance updated: {:?}", msg.balance);
///         }
///         StreamMessage::OrderInfo(msg) => {
///             println!("Order {} status: {:?}", msg.order.id, msg.order.status);
///         }
///         StreamMessage::DltaPoints(msg) => {
///             println!("Points changed: {}", msg.dlta_points.delta);
///         }
///         StreamMessage::Unknown(raw) => {
///             println!("Unknown message: {}", raw);
///         }
///     }
/// }
///
/// // Close the stream when done
/// handle.close().await;
/// ```
pub struct AccountStream {
    ws_url: String,
    api_key: String,
}

impl AccountStream {
    /// Create a new AccountStream instance.
    pub fn new(ws_url: String, api_key: String) -> Self {
        AccountStream { ws_url, api_key }
    }

    /// Subscribe to the account stream and receive real-time updates.
    ///
    /// This is the simple API without automatic reconnection. The stream will
    /// terminate if the connection is lost.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Size of the message buffer (default: 100)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `StreamHandle` - Handle to control the stream (close, check status)
    /// - `mpsc::Receiver<StreamMessage>` - Receiver for stream messages
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection fails.
    pub async fn subscribe(
        &self,
        buffer_size: Option<usize>,
    ) -> Result<(StreamHandle, mpsc::Receiver<StreamMessage>), WError> {
        let buffer = buffer_size.unwrap_or(100);
        let (message_tx, message_rx) = mpsc::channel::<StreamMessage>(buffer);
        let (close_tx, mut close_rx) = mpsc::channel::<()>(1);

        // Build WebSocket URL with authentication
        let ws_endpoint = format!("{}/accounts/ws/stream?api_key={}", self.ws_url, self.api_key);

        // Connect to WebSocket with timeout
        let connect_timeout = Duration::from_secs(30);
        let (ws_stream, _response) = timeout(connect_timeout, connect_async(&ws_endpoint))
            .await
            .map_err(|_| WError::new("AccountStream", "Connection timeout"))?
            .map_err(|e| WError::new("AccountStream", &format!("Connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // Spawn background task to handle WebSocket messages
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Check for close signal
                    _ = close_rx.recv() => {
                        // Send close frame
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                    // Handle incoming messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                let stream_msg = StreamMessage::from_json(&text);
                                if message_tx.send(stream_msg).await.is_err() {
                                    // Receiver dropped, close connection
                                    let _ = write.send(Message::Close(None)).await;
                                    break;
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // Respond to ping with pong
                                if write.send(Message::Pong(data)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                // Server closed connection
                                break;
                            }
                            Some(Err(_)) => {
                                // WebSocket error, close connection
                                break;
                            }
                            None => {
                                // Stream ended
                                break;
                            }
                            _ => {
                                // Ignore other message types (Binary, Pong, Frame)
                            }
                        }
                    }
                }
            }
        });

        let handle = StreamHandle {
            close_tx: Some(close_tx),
        };

        Ok((handle, message_rx))
    }

    /// Subscribe to the account stream with automatic reconnection.
    ///
    /// This method establishes a WebSocket connection and automatically
    /// reconnects using exponential backoff if the connection is lost.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Size of the message buffer (default: 100)
    /// * `reconnect_config` - Configuration for reconnection behavior (default: ReconnectConfig::default())
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `StreamHandle` - Handle to control the stream (close, check status)
    /// - `mpsc::Receiver<StreamEvent>` - Receiver for stream events (messages + lifecycle events)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = ReconnectConfig::default().with_max_retries(5);
    /// let (mut handle, mut receiver) = client.stream
    ///     .subscribe_with_reconnect(None, Some(config)).await?;
    ///
    /// while let Some(event) = receiver.recv().await {
    ///     match event {
    ///         StreamEvent::Message(msg) => { /* handle message */ }
    ///         StreamEvent::Connected => println!("Connected!"),
    ///         StreamEvent::Reconnecting { attempt, delay_ms } => {
    ///             println!("Reconnecting attempt {} in {}ms", attempt, delay_ms);
    ///         }
    ///         StreamEvent::Disconnected { reason } => println!("Disconnected: {}", reason),
    ///         StreamEvent::MaxRetriesExceeded => break,
    ///     }
    /// }
    /// ```
    pub async fn subscribe_with_reconnect(
        &self,
        buffer_size: Option<usize>,
        reconnect_config: Option<ReconnectConfig>,
    ) -> Result<(StreamHandle, mpsc::Receiver<StreamEvent>), WError> {
        let buffer = buffer_size.unwrap_or(100);
        let config = reconnect_config.unwrap_or_default();
        let (event_tx, event_rx) = mpsc::channel::<StreamEvent>(buffer);
        let (close_tx, close_rx) = mpsc::channel::<()>(1);

        let ws_endpoint = format!("{}/accounts/ws/stream?api_key={}", self.ws_url, self.api_key);

        // Spawn the reconnecting stream task
        tokio::spawn(Self::run_reconnecting_stream(
            ws_endpoint,
            config,
            event_tx,
            close_rx,
        ));

        let handle = StreamHandle {
            close_tx: Some(close_tx),
        };

        Ok((handle, event_rx))
    }

    /// Internal: Run a single WebSocket connection, returning why it ended.
    /// The boolean in Error indicates whether connection was ever established.
    async fn run_single_connection(
        ws_endpoint: &str,
        connect_timeout: Duration,
        event_tx: &mpsc::Sender<StreamEvent>,
        close_rx: &mut mpsc::Receiver<()>,
    ) -> ConnectionResult {
        // Attempt to connect with timeout
        let connect_result = timeout(connect_timeout, connect_async(ws_endpoint)).await;

        let ws_stream = match connect_result {
            Ok(Ok((stream, _response))) => stream,
            Ok(Err(e)) => {
                // Never connected
                return ConnectionResult::Error(format!("Connection failed: {}", e), false);
            }
            Err(_) => {
                // Never connected
                return ConnectionResult::Error("Connection timeout".to_string(), false);
            }
        };

        // Notify connected
        if event_tx.send(StreamEvent::Connected).await.is_err() {
            return ConnectionResult::ReceiverDropped;
        }

        let (mut write, mut read) = ws_stream.split();

        // Message loop - connection is now established
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
                            let stream_msg = StreamMessage::from_json(&text);
                            let event = StreamEvent::Message(stream_msg);
                            if event_tx.send(event).await.is_err() {
                                let _ = write.send(Message::Close(None)).await;
                                return ConnectionResult::ReceiverDropped;
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if write.send(Message::Pong(data)).await.is_err() {
                                // Was connected, then failed
                                return ConnectionResult::Error("Failed to send pong".to_string(), true);
                            }
                        }
                        Some(Ok(Message::Close(frame))) => {
                            let reason = frame
                                .map(|f| f.reason.to_string())
                                .unwrap_or_else(|| "Server closed connection".to_string());
                            // Was connected, then server closed
                            return ConnectionResult::Error(reason, true);
                        }
                        Some(Err(e)) => {
                            // Was connected, then error
                            return ConnectionResult::Error(format!("WebSocket error: {}", e), true);
                        }
                        None => {
                            // Was connected, then stream ended
                            return ConnectionResult::Error("Stream ended unexpectedly".to_string(), true);
                        }
                        _ => {
                            // Ignore other message types (Binary, Pong, Frame)
                        }
                    }
                }
            }
        }
    }

    /// Internal: Run the reconnecting stream loop.
    async fn run_reconnecting_stream(
        ws_endpoint: String,
        config: ReconnectConfig,
        event_tx: mpsc::Sender<StreamEvent>,
        mut close_rx: mpsc::Receiver<()>,
    ) {
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
            )
            .await;

            match result {
                ConnectionResult::UserClosed => {
                    // User explicitly closed, don't reconnect
                    break;
                }
                ConnectionResult::ReceiverDropped => {
                    // Receiver dropped, stop the stream
                    break;
                }
                ConnectionResult::Error(reason, was_connected) => {
                    // Reset attempt counter if we had a successful connection
                    // This ensures fresh exponential backoff after a long-running connection drops
                    if was_connected {
                        attempt = 0;
                    }

                    // Send disconnected event
                    let _ = event_tx
                        .send(StreamEvent::Disconnected {
                            reason: reason.clone(),
                        })
                        .await;

                    // Check if we should retry
                    if !config.should_retry(attempt) {
                        let _ = event_tx.send(StreamEvent::MaxRetriesExceeded).await;
                        break;
                    }

                    // Calculate delay and notify
                    let delay_ms = config.delay_for_attempt(attempt);
                    attempt += 1;

                    let _ = event_tx
                        .send(StreamEvent::Reconnecting { attempt, delay_ms })
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

    /// Subscribe with a callback function for each message.
    ///
    /// This is a convenience method that processes messages using a callback
    /// instead of requiring manual message loop handling.
    ///
    /// # Arguments
    ///
    /// * `callback` - Async function called for each received message.
    ///                Return `false` to stop the stream.
    ///
    /// # Returns
    ///
    /// Returns when the stream is closed or the callback returns `false`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// client.stream.subscribe_with_callback(|msg| async move {
    ///     match msg {
    ///         StreamMessage::Balance(b) => println!("Balance: {:?}", b.balance),
    ///         StreamMessage::OrderInfo(o) => println!("Order: {}", o.order.id),
    ///         _ => {}
    ///     }
    ///     true // Continue receiving messages
    /// }).await?;
    /// ```
    pub async fn subscribe_with_callback<F, Fut>(&self, mut callback: F) -> Result<(), WError>
    where
        F: FnMut(StreamMessage) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe(None).await?;

        while let Some(message) = receiver.recv().await {
            if !callback(message).await {
                break;
            }
        }

        handle.close().await;
        Ok(())
    }

    /// Subscribe with a callback function and automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `reconnect_config` - Configuration for reconnection behavior
    /// * `callback` - Async function called for each event. Return `false` to stop.
    pub async fn subscribe_with_reconnect_callback<F, Fut>(
        &self,
        reconnect_config: Option<ReconnectConfig>,
        mut callback: F,
    ) -> Result<(), WError>
    where
        F: FnMut(StreamEvent) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let (mut handle, mut receiver) = self.subscribe_with_reconnect(None, reconnect_config).await?;

        while let Some(event) = receiver.recv().await {
            if !callback(event).await {
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
    fn test_stream_message_parsing() {
        // Test balance message parsing
        let balance_json = r#"{
            "type": "Account",
            "sub_type": "balance",
            "balance": [
                {"asset": "ADA", "asset_unit": "lovelace", "free": "1000000", "locked": "0"}
            ]
        }"#;

        let msg = StreamMessage::from_json(balance_json);
        assert!(msg.is_balance());
        assert!(!msg.is_order_info());

        if let StreamMessage::Balance(balance_msg) = msg {
            assert_eq!(balance_msg.balance.len(), 1);
            assert_eq!(balance_msg.balance[0].asset, "ADA");
        }

        // Test order info message parsing
        let order_json = r#"{
            "type": "Account",
            "sub_type": "order_info",
            "order": {
                "id": "order-123",
                "account_id": "acc-456",
                "status": "open",
                "symbol": "ADAUSDM",
                "base_qty": "100",
                "quote_qty": "50",
                "side": "buy",
                "price": "0.5",
                "type": "limit",
                "locked_base_qty": "0",
                "locked_quote_qty": "50",
                "executed_base_qty": "0",
                "executed_quote_qty": "0",
                "ob_open_order_base_qty": "100",
                "commission_unit": "USDM",
                "commission": "0",
                "commission_rate_bp": 30,
                "executed_price": "0",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        }"#;

        let msg = StreamMessage::from_json(order_json);
        assert!(msg.is_order_info());

        if let StreamMessage::OrderInfo(order_msg) = msg {
            assert_eq!(order_msg.order.id, "order-123");
            assert_eq!(order_msg.order.symbol, "ADAUSDM");
        }

        // Test DLTA points message parsing
        let points_json = r#"{
            "type": "Account",
            "sub_type": "dlta_points",
            "dlta_points": {
                "delta": "100",
                "new_total": "1000",
                "season_points": "500",
                "source_type": "trade",
                "source_ref": "order-123",
                "league": "gold"
            }
        }"#;

        let msg = StreamMessage::from_json(points_json);
        assert!(msg.is_dlta_points());

        if let StreamMessage::DltaPoints(points_msg) = msg {
            assert_eq!(points_msg.dlta_points.delta, "100");
            assert_eq!(points_msg.dlta_points.league, "gold");
        }

        // Test unknown message
        let unknown_json = r#"{"type": "Unknown", "data": "test"}"#;
        let msg = StreamMessage::from_json(unknown_json);
        assert!(msg.is_unknown());
    }

    #[test]
    fn test_reconnect_config_defaults() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_retries, Some(10));
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 60000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.connect_timeout_ms, 30000);
        assert_eq!(config.jitter_factor, 0.25);
    }

    #[test]
    fn test_reconnect_config_no_reconnect() {
        let config = ReconnectConfig::no_reconnect();
        assert_eq!(config.max_retries, Some(0));
        assert!(!config.should_retry(0));
    }

    #[test]
    fn test_reconnect_config_infinite() {
        let config = ReconnectConfig::infinite();
        assert_eq!(config.max_retries, None);
        assert!(config.should_retry(0));
        assert!(config.should_retry(100));
        assert!(config.should_retry(1000));
    }

    #[test]
    fn test_reconnect_config_builder() {
        let config = ReconnectConfig::default()
            .with_max_retries(5)
            .with_initial_delay_ms(500)
            .with_max_delay_ms(30000)
            .with_backoff_multiplier(1.5)
            .with_connect_timeout_ms(10000)
            .with_jitter_factor(0.1);

        assert_eq!(config.max_retries, Some(5));
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.connect_timeout_ms, 10000);
        assert_eq!(config.jitter_factor, 0.1);

        // Test jitter_factor clamping
        let config_clamped = ReconnectConfig::default().with_jitter_factor(1.5);
        assert_eq!(config_clamped.jitter_factor, 1.0);

        let config_clamped_neg = ReconnectConfig::default().with_jitter_factor(-0.5);
        assert_eq!(config_clamped_neg.jitter_factor, 0.0);
    }

    #[test]
    fn test_reconnect_config_delay_calculation() {
        let config = ReconnectConfig::default()
            .with_initial_delay_ms(1000)
            .with_max_delay_ms(60000)
            .with_backoff_multiplier(2.0);

        // Test base delay (deterministic, no jitter)
        // Exponential backoff: 1000 * 2^attempt
        assert_eq!(config.base_delay_for_attempt(0), 1000);   // 1000 * 2^0 = 1000
        assert_eq!(config.base_delay_for_attempt(1), 2000);   // 1000 * 2^1 = 2000
        assert_eq!(config.base_delay_for_attempt(2), 4000);   // 1000 * 2^2 = 4000
        assert_eq!(config.base_delay_for_attempt(3), 8000);   // 1000 * 2^3 = 8000
        assert_eq!(config.base_delay_for_attempt(4), 16000);  // 1000 * 2^4 = 16000
        assert_eq!(config.base_delay_for_attempt(5), 32000);  // 1000 * 2^5 = 32000
        assert_eq!(config.base_delay_for_attempt(6), 60000);  // Capped at max_delay_ms
        assert_eq!(config.base_delay_for_attempt(10), 60000); // Still capped
    }

    #[test]
    fn test_reconnect_config_jitter_range() {
        let config = ReconnectConfig::default()
            .with_initial_delay_ms(1000)
            .with_max_delay_ms(60000)
            .with_backoff_multiplier(2.0)
            .with_jitter_factor(0.25);

        // With 25% jitter, delay should be within ±25% of base
        // For attempt 0: base = 1000, range = [750, 1250]
        for _ in 0..100 {
            let delay = config.delay_for_attempt(0);
            assert!(delay >= 750 && delay <= 1250, "delay {} out of expected range [750, 1250]", delay);
        }

        // Test with no jitter
        let config_no_jitter = ReconnectConfig::default()
            .with_initial_delay_ms(1000)
            .with_jitter_factor(0.0);

        assert_eq!(config_no_jitter.delay_for_attempt(0), 1000);
        assert_eq!(config_no_jitter.delay_for_attempt(1), 2000);
    }

    #[test]
    fn test_reconnect_config_should_retry() {
        let config = ReconnectConfig::default().with_max_retries(3);

        assert!(config.should_retry(0));
        assert!(config.should_retry(1));
        assert!(config.should_retry(2));
        assert!(!config.should_retry(3));
        assert!(!config.should_retry(4));
    }

    #[test]
    fn test_stream_event_variants() {
        // Test that all StreamEvent variants can be constructed
        let _connected = StreamEvent::Connected;
        let _disconnected = StreamEvent::Disconnected {
            reason: "test".to_string(),
        };
        let _reconnecting = StreamEvent::Reconnecting {
            attempt: 1,
            delay_ms: 1000,
        };
        let _max_retries = StreamEvent::MaxRetriesExceeded;

        // Message variant requires a StreamMessage
        let balance_json = r#"{
            "type": "Account",
            "sub_type": "balance",
            "balance": []
        }"#;
        let msg = StreamMessage::from_json(balance_json);
        let _message_event = StreamEvent::Message(msg);
    }
}
