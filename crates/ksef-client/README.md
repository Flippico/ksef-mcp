# KSeF Client

Rust client library for the Polish e-invoicing system KSeF (Krajowy System e-Faktur) API.

## Overview

This library provides a convenient async interface for interacting with the KSeF API. It can be used standalone in any Rust project that needs to integrate with the Polish e-invoicing system.

## Features

- Async/await API using Tokio
- Authentication session management
- Invoice operations (query, retrieve, export, submit)
- Online session management
- Certificate retrieval
- Rate limit monitoring
- Configurable API base URL

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ksef-client = { path = "../path/to/ksef_mcp/crates/ksef-client" }
tokio = { version = "1", features = ["full"] }
```

## Usage

### Basic Example

```rust
use ksef_client::KsefClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KsefClient::new();

    // Get public certificates (doesn't require auth)
    let certs = client.get_public_key_certificates().await?;
    println!("Certificates: {}", certs);

    Ok(())
}
```

### With Authentication

```rust
use ksef_client::KsefClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KsefClient::new();

    // Set session token from your authentication flow
    client.set_session_token("your-session-token".to_string());

    // Get active sessions
    let sessions = client.get_active_sessions(10, None).await?;
    println!("Active sessions: {}", sessions);

    // Get current session info
    let current = client.get_current_session().await?;
    println!("Current session: {}", current);

    Ok(())
}
```

### Custom API Endpoint

```rust
use ksef_client::KsefClient;

let client = KsefClient::with_base_url(
    "https://your-custom-endpoint.com/v2".to_string()
);
```

## API Methods

### Authentication & Sessions

- `get_active_sessions(page_size, continuation_token)` - List active sessions
- `get_current_session()` - Get current session details
- `terminate_session(reference_number)` - Terminate a session

### Invoice Operations

- `get_invoice(ksef_number)` - Get invoice by KSeF number
- `query_invoice_metadata(query)` - Query invoice metadata
- `create_invoice_export(params)` - Create an export
- `get_export_status(reference_number)` - Get export status

### Session Management

- `create_online_session(params)` - Create an online session
- `close_online_session(reference_number)` - Close a session
- `submit_invoice(session_ref, invoice_data)` - Submit invoice XML

### System

- `get_public_key_certificates()` - Get Ministry of Finance certificates
- `get_rate_limits()` - Get current rate limits

### Token Management

- `set_session_token(token)` - Set authentication token
- `clear_session_token()` - Clear authentication token

## Environment

By default, the client connects to the KSeF test environment:
- `https://api-test.ksef.mf.gov.pl/v2`

For production use, create a client with the production URL.

## Error Handling

All methods return `Result<String, anyhow::Error>`. API errors include the HTTP status code and response body.

```rust
match client.get_invoice("invalid-number").await {
    Ok(invoice) => println!("Invoice: {}", invoice),
    Err(e) => eprintln!("Error: {}", e),
}
```

## License

This project is provided as-is for integration with the Polish e-invoicing system KSeF.
