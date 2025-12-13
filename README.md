# payrust

[![Crates.io](https://img.shields.io/crates/v/payrust.svg)](https://crates.io/crates/payrust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PayPal REST API client for Rust.

## What it does

- Create and capture orders
- Process refunds (full or partial)
- Verify webhook signatures
- Auto-refresh access tokens

## Installation

```toml
[dependencies]
payrust = "0.1"
```

## Usage

```rust
use payrust::prelude::*;

#[tokio::main]
async fn main() -> payrust::Result<()> {
    let client = PayPal::sandbox("CLIENT_ID", "SECRET").await?;

    let order = client.create_order()
        .amount(29.99, Currency::USD)
        .item("Product", 1, 29.99)
        .return_url("https://example.com/success")
        .cancel_url("https://example.com/cancel")
        .send()
        .await?;

    println!("{}", order.approve_url().unwrap());

    // after approval
    let captured = client.capture(&order.id).await?;

    Ok(())
}
```

## License

MIT
