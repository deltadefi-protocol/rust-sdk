# DeltaDeFi Rust SDK

[![Crates.io](https://img.shields.io/crates/v/deltadefi.svg)](https://crates.io/crates/deltadefi)
[![Documentation](https://docs.rs/deltadefi/badge.svg)](https://docs.rs/deltadefi)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The official Rust SDK for DeltaDeFi protocol, providing a comprehensive and ergonomic interface for decentralized finance operations. This SDK enables developers to interact with DeltaDeFi's trading platform, manage accounts, execute orders, and handle wallet operations seamlessly.

## 🚀 Features

- **🔐 Wallet Management**: Secure wallet integration with master and operation key support
- **📊 Order Management**: Complete order lifecycle - place, cancel, and track orders
- **💰 Account Operations**: Deposits, withdrawals, transfers, and balance management
- **📈 Market Data**: Real-time market prices and historical aggregated price data
- **🌐 Multi-Environment**: Support for Mainnet, Staging, Dev, and custom environments
- **⚡ Async/Await**: Fully asynchronous API built on tokio
- **🛡️ Type Safety**: Comprehensive type definitions with serde serialization
- **🔒 Security**: Built-in transaction signing and encryption support

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
deltadefi = "0.1.0"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
dotenv = "0.15" # For environment variable management
```

## 🏁 Quick Start

```rust
use deltadefi::{DeltaDeFi, Stage, OrderSide, OrderType};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize the client
    let api_key = env::var("DELTADEFI_API_KEY")?;
    let mut client = DeltaDeFi::new(api_key, Stage::Staging, None)?;

    // Load operation key for signing transactions
    let password = env::var("ENCRYPTION_PASSCODE")?;
    client.load_operation_key(&password).await?;

    // Place a limit order
    let order_result = client.post_order(
        "ADAUSDM",
        OrderSide::Buy,
        OrderType::Limit,
        100.0,          // quantity
        Some(1.25),     // price
        None,           // limit_slippage
        None,           // max_slippage_basis_point
    ).await?;

    println!("Order placed: {}", order_result.order_id);

    Ok(())
}
```

## 🔧 Configuration

### Environment Setup

Create a `.env` file with your credentials:

```env
DELTADEFI_API_KEY=your_api_key_here
ENCRYPTION_PASSCODE=your_encryption_password
```

### Network Environments

```rust
use deltadefi::{DeltaDeFi, Stage};

// Different network configurations
let mainnet_client = DeltaDeFi::new(api_key, Stage::Mainnet, None)?;
let staging_client = DeltaDeFi::new(api_key, Stage::Staging, None)?;
```

## 📚 API Reference

### DeltaDeFi Client

The main client provides access to all SDK functionality:

```rust
pub struct DeltaDeFi {
    pub accounts: Accounts,  // Account management
    pub market: Market,     // Market data
    pub order: Order,       // Order operations
    // ... wallet fields
}
```

#### Core Methods

```rust
impl DeltaDeFi {
    // Initialize client
    pub fn new(api_key: String, network: Stage, master_key: Option<WalletType>) -> Result<Self, WError>

    // Load operation key for transaction signing
    pub async fn load_operation_key(&mut self, password: &str) -> Result<(), WError>

    // Convenience method for placing orders
    pub async fn post_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        limit_slippage: Option<bool>,
        max_slippage_basis_point: Option<u64>,
    ) -> Result<SubmitPlaceOrderTransactionResponse, WError>

    // Convenience method for canceling orders
    pub async fn cancel_order(&self, order_id: &str) -> Result<(), WError>

    // Transaction signing
    pub fn sign_tx_by_master_key(&self, tx: &str) -> Result<String, WError>
    pub fn sign_tx_by_operation_key(&self, tx: &str) -> Result<String, WError>
}
```

### Accounts Module

Manage account operations, balances, and transaction history:

```rust
impl Accounts {
    // Get encrypted operation key
    pub async fn get_operation_key(&self) -> Result<GetOperationKeyResponse, WError>

    // Generate new API key
    pub async fn create_new_api_key(&self) -> Result<CreateNewAPIKeyResponse, WError>

    // Get account balance
    pub async fn get_account_balance(&self) -> Result<GetAccountBalanceResponse, WError>

    // Transaction history
    pub async fn get_deposit_records(&self) -> Result<GetDepositRecordsResponse, WError>
    pub async fn get_withdrawal_records(&self) -> Result<GetWithdrawalRecordsResponse, WError>
    pub async fn get_order_records(
        &self,
        status: OrderRecordStatus,
        limit: Option<u32>,
        page: Option<u32>,
        symbol: Option<String>,
    ) -> Result<GetOrderRecordsResponse, WError>

    // Transaction building and submission
    pub async fn build_deposit_transaction(
        &self,
        deposit_amount: Vec<Asset>,
        input_utxos: Vec<UTxO>,
    ) -> Result<BuildDepositTransactionResponse, WError>

    pub async fn submit_deposit_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitDepositTransactionResponse, WError>

    // Similar methods for withdrawal and transferal transactions...
}
```

### Market Module

Access market data and price information:

```rust
impl Market {
    // Get current market price
    pub async fn get_market_price(&self, symbol: &str) -> Result<GetMarketPriceResponse, WError>

    // Get historical price data
    pub async fn get_aggregated_price(
        &self,
        symbol: Symbol,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<GetAggregatedPriceResponse, WError>
}
```

### Order Module

Handle order lifecycle operations:

```rust
impl Order {
    // Build order transaction
    pub async fn build_place_order_transaction(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        limit_slippage: Option<bool>,
        max_slippage_basis_point: Option<u64>,
    ) -> Result<BuildPlaceOrderTransactionResponse, WError>

    // Submit order transaction
    pub async fn submit_place_order_transaction(
        &self,
        order_id: &str,
        signed_tx: &str,
    ) -> Result<SubmitPlaceOrderTransactionResponse, WError>

    // Build cancel order transaction
    pub async fn build_cancel_order_transaction(
        &self,
        order_id: &str,
    ) -> Result<BuildCancelOrderTransactionResponse, WError>

    // Submit cancel order transaction
    pub async fn submit_cancel_order_transaction(&self, signed_tx: &str) -> Result<(), WError>
}
```

## 🏷️ Types and Enums

### Core Types

```rust
// Trading symbols
pub enum Symbol {
    ADAUSDM,
}

// Time intervals for price data
pub enum Interval {
    Interval5m,   // 5 minutes
    Interval15m,  // 15 minutes
    Interval30m,  // 30 minutes
    Interval1h,   // 1 hour
    Interval1d,   // 1 day
}

// Order types
pub enum OrderType {
    Market,
    Limit,
}

// Order sides
pub enum OrderSide {
    Buy,
    Sell,
}

// Order statuses
pub enum OrderStatus {
    Processing,
    Open,
    FullyFilled,
    PartiallyFilled,
    Cancelled,
    PartiallyCancelled,
    Failed,
}

// Transaction statuses
pub enum TransactionStatus {
    Building,
    HeldForOrder,
    Submitted,
    SubmissionFailed,
    Confirmed,
}
```

## 💡 Examples

### Complete Order Management

```rust
use deltadefi::{DeltaDeFi, Stage, OrderSide, OrderType, OrderRecordStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = setup_client().await?;

    // Place a market order with slippage protection
    let market_order = client.post_order(
        "ADAUSDM",
        OrderSide::Buy,
        OrderType::Market,
        50.0,
        None,                    // No price for market order
        Some(true),             // Enable slippage protection
        Some(100),              // Max 1% slippage (100 basis points)
    ).await?;

    println!("Market order placed: {}", market_order.order_id);

    // Place a limit order
    let limit_order = client.post_order(
        "ADAUSDM",
        OrderSide::Sell,
        OrderType::Limit,
        75.0,
        Some(1.30),             // Limit price
        None,
        None,
    ).await?;

    println!("Limit order placed: {}", limit_order.order_id);

    // Check order history
    let order_history = client.accounts.get_order_records(
        OrderRecordStatus::OrderHistory,
        Some(10),               // Limit to 10 records
        Some(1),                // First page
        None,                   // All symbols
    ).await?;

    println!("Order history: {} orders", order_history.total_count);

    // Cancel the limit order
    client.cancel_order(&limit_order.order_id).await?;
    println!("Limit order cancelled");

    Ok(())
}

async fn setup_client() -> Result<DeltaDeFi, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let api_key = std::env::var("DELTADEFI_API_KEY")?;
    let password = std::env::var("ENCRYPTION_PASSCODE")?;

    let mut client = DeltaDeFi::new(api_key, Stage::Staging, None)?;
    client.load_operation_key(&password).await?;

    Ok(client)
}
```

### Account Management

```rust
use deltadefi::{DeltaDeFi, Stage};
use whisky::{Asset, UTxO};

async fn manage_account(client: &DeltaDeFi) -> Result<(), Box<dyn std::error::Error>> {
    // Check account balance
    let balance = client.accounts.get_account_balance().await?;
    println!("Account balances:");
    for asset_balance in balance.balances {
        println!("  {}: {} free, {} locked",
                 asset_balance.asset,
                 asset_balance.free,
                 asset_balance.locked);
    }

    // Get deposit records
    let deposits = client.accounts.get_deposit_records().await?;
    println!("Recent deposits: {}", deposits.len());

    // Build and submit a withdrawal
    let withdrawal_amount = vec![Asset {
        unit: "lovelace".to_string(),
        quantity: 1000000.0, // 1 ADA in lovelace
    }];

    let withdrawal_tx = client.accounts
        .build_withdrawal_transaction(
            BuildWithdrawalTransactionRequest::new(withdrawal_amount),
        )
        .await?;

    // Sign the transaction
    let signed_tx = client.sign_tx_by_master_key(&withdrawal_tx.tx_hex)?;

    // Submit the withdrawal
    let withdrawal_result = client.accounts
        .submit_withdrawal_transaction(&signed_tx)
        .await?;

    println!("Withdrawal submitted: {}", withdrawal_result.tx_hash);

    Ok(())
}
```

### Market Data Analysis

```rust
use deltadefi::{DeltaDeFi, Stage, Symbol, Interval};
use chrono::{Utc, Duration};

async fn analyze_market_data(client: &DeltaDeFi) -> Result<(), Box<dyn std::error::Error>> {
    // Get current market price
    let current_price = client.market.get_market_price("ADAUSDM").await?;
    println!("Current ADAUSDM price: {}", current_price.price);

    // Get historical data for the last 24 hours
    let now = Utc::now();
    let yesterday = now - Duration::hours(24);

    let historical_data = client.market.get_aggregated_price(
        Symbol::ADAUSDM,
        Interval::Interval1h,
        yesterday.timestamp(),
        now.timestamp(),
    ).await?;

    println!("Historical data points: {}", historical_data.data.len());

    // Calculate price change
    if let (Some(first), Some(last)) = (historical_data.data.first(), historical_data.data.last()) {
        let price_change = ((last.close - first.open) / first.open) * 100.0;
        println!("24h price change: {:.2}%", price_change);
    }

    Ok(())
}
```

## 🛡️ Error Handling

The SDK uses the `WError` type for comprehensive error handling:

```rust
use whisky::WError;

// Handle specific errors
match client.post_order(/* ... */).await {
    Ok(response) => println!("Order successful: {}", response.order_id),
    Err(WError { source, message }) => {
        eprintln!("Order failed - Source: {}, Message: {}", source, message);

        // Handle specific error types
        match source.as_str() {
            "build_place_order_transaction" => {
                // Handle order building errors
                if message.contains("Missing required parameter") {
                    eprintln!("Please check order parameters");
                }
            }
            "DeltaDeFi - post - send_request" => {
                // Handle network errors
                eprintln!("Network error, please retry");
            }
            _ => eprintln!("Unexpected error occurred"),
        }
    }
}
```

## 🔐 Security Best Practices

1. **Environment Variables**: Store sensitive data in environment variables
2. **Key Management**: Never hardcode private keys or passwords
3. **Network Security**: Use HTTPS endpoints for all API calls
4. **Error Handling**: Avoid exposing sensitive information in error messages

```rust
// ✅ Good - Using environment variables
let api_key = env::var("DELTADEFI_API_KEY")?;

// ❌ Bad - Hardcoded credentials
let api_key = "your-api-key-here";
```

## 📄 License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [DeltaDeFi Protocol](https://deltadefi.io)
- [SDKs Demo Repository](https://github.com/deltadefi-protocol/sdks-demo/tree/main/rust)
- [API Documentation](https://docs.rs/deltadefi)
- [Crates.io Package](https://crates.io/crates/deltadefi)

## 📞 Support

- GitHub Issues: [Report bugs or request features](https://github.com/deltadefi-protocol/rust-sdk/issues)
- Telegram community: [Telegram Community](https://t.me/deltadefi_community)
- Discord community: [Dicord Community](https://discord.gg/55Z25r3QfC)
