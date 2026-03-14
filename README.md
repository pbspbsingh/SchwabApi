# schwab-api

An idiomatic, async-first Rust client for the [Charles Schwab brokerage API](https://developer.schwab.com).
Streaming is the primary use case; the full REST API is also covered.

## Features

- **Fully typed** — every request parameter and response field is a named Rust type. No raw `serde_json::Value` in the public surface.
- **Async throughout** — `tokio` runtime, `reqwest` for HTTP, `tokio-tungstenite` for WebSocket.
- **Transparent reconnect** — streaming reconnects automatically with exponential backoff. Callers never see a dropped connection; their channel handles stay valid.
- **Back-pressure** — bounded `tokio::sync::mpsc` channels. Slow consumers slow down the wire, non-lossy.
- **Single streaming connection** — `StreamClient::connect()` returns `Arc<StreamClient>`. Schwab allows one streaming connection per account; the connection tears down when the last `Arc` is dropped.
- **Opaque auth** — `TokenManager::create()` is the only public entry point. It handles token file loading, silent refresh, and the full OAuth flow internally.

## Quick start

### Prerequisites

1. Register an application at [developer.schwab.com](https://developer.schwab.com) and note your **app key** and **app secret**.
2. Set the redirect URI to `https://127.0.0.1:8443` (or another `https://127.0.0.1` address).
3. Generate a self-signed TLS certificate for the local callback server:

```sh
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem \
    -days 365 -nodes -subj "/CN=127.0.0.1"
```

### Environment variables

```sh
export SCHWAB_APP_KEY=your_app_key
export SCHWAB_APP_SECRET=your_app_secret
export SCHWAB_REDIRECT_URI=https://127.0.0.1:8443   # default
export SCHWAB_TLS_CERT=cert.pem                      # default
export SCHWAB_TLS_KEY=key.pem                        # default
```

### Run the demo

```sh
cargo run
```

On first run, a URL is printed. Open it in your browser, authorize, and the callback is handled automatically. Tokens are saved to `tokens.json` and reused on subsequent runs.

## Usage

```rust
use std::{path::Path, sync::Arc};
use schwab_api::auth::{OAuthConfig, TokenManager};
use schwab_api::stream::fields::LevelOneEquityField;
use schwab_api::{SchwabClient, StreamClient};

#[tokio::main]
async fn main() -> schwab_api::Result<()> {
    let config = OAuthConfig {
        app_key:       std::env::var("SCHWAB_APP_KEY").unwrap(),
        app_secret:    std::env::var("SCHWAB_APP_SECRET").unwrap(),
        redirect_uri:  "https://127.0.0.1:8443".to_string(),
        tls_cert_path: "cert.pem".into(),
        tls_key_path:  "key.pem".into(),
    };

    // Loads tokens.json if valid; otherwise runs the OAuth flow.
    let tokens = TokenManager::create(config, Path::new("tokens.json")).await?;

    // REST — snapshot quote
    let client = SchwabClient::new(Arc::clone(&tokens));
    let quotes = client.get_quotes(&["AAPL"], None, None).await?;

    // Streaming — subscribe to L1 equity prices
    let prefs = client.get_user_preferences().await?;
    let stream = StreamClient::connect(Arc::clone(&tokens), prefs).await?;

    let mut rx = stream
        .level_one_equities()
        .subscribe(&["AAPL", "MSFT"], &LevelOneEquityField::all(), 256)
        .await?;

    while let Some(event) = rx.recv().await {
        println!("{}: bid={:?}  ask={:?}", event.symbol, event.bid_price, event.ask_price);
    }

    Ok(())
}
```

## REST API

All methods on `SchwabClient` are `async` and return `Result<T>`.

### Accounts

```rust
let numbers = client.get_account_numbers().await?;          // Vec<AccountNumber>
let account = client.get_account(&hash, None).await?;       // Account
let accounts = client.get_accounts(None).await?;            // Vec<Account>
let prefs = client.get_user_preferences().await?;           // UserPreferences
```

### Quotes

```rust
let quote  = client.get_quote("AAPL", None).await?;
let quotes = client.get_quotes(&["AAPL", "MSFT"], None, None).await?;
```

### Price history

```rust
let bars = client.get_price_history_every_minute("AAPL", None, None, None, None, None).await?;
let bars = client.get_price_history_every_day("AAPL", None, None, None, None, None).await?;
```

### Options

```rust
let chain = client.get_option_chain(req).await?;
```

### Orders

```rust
let order_id = client.place_order(&hash, &order).await?;
let order    = client.get_order(&hash, order_id).await?;
client.cancel_order(&hash, order_id).await?;
```

### Market data

```rust
let movers = client.get_movers("$SPX", None, None).await?;
let hours  = client.get_market_hours(&["equity"], None).await?;
```

## Streaming services

Subscribe to any of the 13 services. The first `subscribe()` call returns a `Receiver`; subsequent calls expand the symbol set without creating a new channel.

| Service | Method | Event type |
|---|---|---|
| Level 1 equities | `level_one_equities()` | `LevelOneEquityEvent` |
| Level 1 options | `level_one_options()` | `LevelOneOptionEvent` |
| Level 1 futures | `level_one_futures()` | `LevelOneFuturesEvent` |
| Level 1 forex | `level_one_forex()` | `LevelOneForexEvent` |
| Level 1 futures options | `level_one_futures_options()` | `LevelOneFuturesOptionsEvent` |
| Chart equity (1-min bars) | `chart_equity()` | `ChartEquityEvent` |
| Chart futures (1-min bars) | `chart_futures()` | `ChartFuturesEvent` |
| NYSE Level 2 book | `nyse_book()` | `BookEvent` |
| NASDAQ Level 2 book | `nasdaq_book()` | `BookEvent` |
| Options Level 2 book | `options_book()` | `BookEvent` |
| Equity screener | `screener_equity()` | `ScreenerEvent` |
| Option screener | `screener_option()` | `ScreenerEvent` |
| Account activity | `account_activity()` | `AccountActivityEvent` |

```rust
// Subscribe
let mut rx = stream.level_one_equities()
    .subscribe(&["AAPL"], &[LevelOneEquityField::BidPrice, LevelOneEquityField::AskPrice], 128)
    .await?;

// Add more symbols (no new channel)
stream.level_one_equities()
    .add_symbols(&["MSFT"], &[LevelOneEquityField::BidPrice, LevelOneEquityField::AskPrice])
    .await?;

// Unsubscribe
stream.level_one_equities().unsubscribe(&["AAPL"]).await?;
```

### Streaming events

All fields except `symbol` are `Option<T>` — Schwab sends **partial updates** only. If you need a full state snapshot for a symbol, merge incoming events into your own state map.

Millisecond timestamp fields (e.g. `quote_time_millis`) are decoded to `DateTime<Utc>`.

### Transparent reconnect

If the WebSocket drops or goes silent for 15 s (Schwab sends a heartbeat every ~10 s), the library reconnects automatically with exponential backoff (1 s → 2 s → 4 s → 8 s → 16 s → 30 s cap, with jitter). All subscriptions are replayed on reconnect. A `tracing::warn!` is emitted during the gap.

## Authentication details

`TokenManager::create(config, path)` is the only entry point:

- If `path` exists and the refresh token is still valid → tokens are loaded silently.
- Otherwise → the OAuth flow runs:
  1. A local HTTPS server is bound on the host:port from `redirect_uri`.
  2. The authorize URL is printed to stdout.
  3. Open the URL in your browser, log in to Schwab, and approve.
  4. Schwab redirects to your local server. The code is extracted automatically.
  5. Tokens are exchanged and saved to `path`.

Access tokens (30 min lifetime) are refreshed automatically and silently. When within 60 s of expiry, `get_valid_token()` acquires a write lock, refreshes via `refresh_token`, and saves the new tokens to disk in a background task.

Refresh tokens (7-day lifetime) require repeating the browser flow when they expire.

## Error handling

```rust
pub enum Error {
    Http(reqwest::Error),
    WebSocket(tungstenite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    OAuth { code: String, message: String },
    StreamLoginFailed { code: i64, msg: String },
    UnexpectedStreamResponse { requestid: String },
    SubscriptionFailed { code: i64, msg: String },
    AlreadySubscribed { service: String },
    NotSubscribed { service: String },
    StreamDisconnected,
    TokenExpired,
    Api { status: u16, body: String },
}
```

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `reqwest` (rustls-tls) | HTTP REST client |
| `tokio-tungstenite` (rustls) | WebSocket client |
| `tokio-rustls` + `rustls-pemfile` | Local TLS callback server |
| `serde` + `serde_json` | JSON serialization |
| `chrono` | Timestamp handling |
| `thiserror` | Error type derivation |
| `tracing` | Structured logging |

## License

MIT
