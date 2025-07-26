# DeltaDeFi Rust SDK

The DeltaDeFi Rust SDK provides a convenient way to interact with the DeltaDeFi protocol. It allows developers to perform operations such as placing and canceling orders, managing wallets, and more.

## Features

- **Order Management**: Build and submit transactions for placing and canceling orders.
- **Wallet Integration**: Load operation keys and sign transactions.
- **Environment Support**: Easily switch between Mainnet, Staging, and Dev environments.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
deltadefi = "<latest-version>"
```

## Usage

Here is an example of how to use the DeltaDeFi Rust SDK to place and cancel an order:

```rust
use deltadefi::{DeltaDeFi, OrderSide, OrderType, Stage};
use dotenv::dotenv;
use std::env;

pub async fn order() {
    dotenv().ok();
    let api_key = env::var("DELTADEFI_API_KEY").expect("DELTADEFI_API_KEY must be set");
    let encryption_passcode =
        env::var("ENCRYPTION_PASSCODE").expect("ENCRYPTION_PASSCODE must be set");

    // Initialize DeltaDeFi client and wallet
    let mut deltadefi = DeltaDeFi::new(api_key, Stage::Staging, None);
    deltadefi
        .load_operation_key(&encryption_passcode)
        .await
        .unwrap();

    // Build place order transaction
    let res = deltadefi
        .post_order(
            "ADAUSDX",
            OrderSide::Sell,
            OrderType::Limit,
            51.0,
            Some(1.5),
            None,
            None,
        )
        .await
        .expect("Failed to build place order transaction");

    println!("Order submitted successfully: {:?}", res);

    let res = deltadefi
        .order
        .build_cancel_order_transaction(&res.order_id)
        .await
        .expect("Failed to build cancel order transaction");

    println!("Order canceled successfully: {:?}", res);
}
```

## Example Usage

For more examples, visit the [DeltaDeFi SDKs Demo Repository](https://github.com/deltadefi-protocol/sdks-demo/tree/main/rust).

## Contributing

We welcome contributions! Please follow our [Contributing Guide](CONTRIBUTING.md) before submitting a pull request.

## License

This project is licensed under the Apache-2.0 License.
