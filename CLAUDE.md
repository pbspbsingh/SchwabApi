# SchwabApi — Rust Client

## Project Overview

An idiomatic, async-first Rust client for the Charles Schwab brokerage API, based on analysis
of the `schwab-py` reference implementation. Streaming is the primary use case.

---

## Status

Complete. Compiles cleanly (`cargo check` — zero errors, zero warnings).
All streaming field IDs verified against `schwab-py` (alexgolec/schwab-py).

### What is built

| File | Status |
|------|--------|
| `src/error.rs` | ✅ Unified `Error` enum + `Result<T>` alias |
| `src/auth.rs` | ✅ `OAuthConfig`, `TokenManager::create()` — opaque, self-contained |
| `src/client.rs` | ✅ `SchwabClient` — full REST API |
| `src/stream/mod.rs` | ✅ `StreamClient` — connect, recv_loop, reconnect, heartbeat watchdog, logout |
| `src/stream/protocol.rs` | ✅ Wire types (internal) |
| `src/stream/fields.rs` | ✅ All 10 field enums — verified against current Schwab API |
| `src/stream/services.rs` | ✅ All 13 streaming services |
| `src/models/` | ✅ All REST models |
| `src/models/stream/` | ✅ All streaming event structs — field IDs match current Schwab API |
| `src/main.rs` | ✅ Demo: AAPL quote + L1 streaming |
| `README.md` | ✅ Full usage documentation |

### Key design decisions made during implementation

- **`TokenManager` is fully opaque** — single entry point `TokenManager::create(config, path)`
  handles file load, expiry check, and full OAuth flow internally. No public methods.
- **OAuth callback is a local HTTPS server** — `OAuthConfig` takes `tls_cert_path` / `tls_key_path`
  (user-provided PEM files). On first run the authorize URL is printed; the server listens for
  Schwab's redirect and extracts the `code` automatically.
- **`StreamClient::connect()` returns `Arc<StreamClient>`** — Schwab allows one streaming
  connection per account. Connection is torn down when the last `Arc` is dropped (`Drop` impl).
- **Streaming subscription API**: `subscribe()` returns `mpsc::Receiver` (first call only),
  `add_symbols()` expands the subscription server-side, `unsubscribe()` shrinks it.
- **All file I/O is `tokio::fs`** — no `std::fs` in async functions.
- **`Error::Io`** added to cover `std::io::Error` from the TLS callback server.
- **Heartbeat watchdog** — if no message received for 15 s (Schwab heartbeats every ~10 s),
  connection is assumed stuck, torn down, and retried via the reconnect loop.
- **Streaming field IDs match current Schwab API** — verified against `schwab-py`. Original
  implementation was based on old TDA field numbering; corrected for `LevelOneEquityField`
  (fields 10–51), `LevelOneOptionField` (fields 11–55), `LevelOneFuturesField` (fields 6–7
  swapped), `LevelOneFuturesOptionField` (fields 15–31), and `ChartEquityField` (fields 1–6).

---

## Design Principles

1. **Typed everything** — every request parameter and response field is a named Rust type (struct/enum). No generic `Value` or `HashMap<String, serde_json::Value>` in the public API.
2. **Async-first** — tokio throughout; `reqwest` for HTTP, `tokio-tungstenite` for WebSocket. All file I/O via `tokio::fs`.
3. **Streaming via bounded mpsc** — `tokio::sync::mpsc::channel` with a caller-specified capacity. No broadcast (lossy). Back-pressure propagates naturally.
4. **Builder pattern for requests** — optional fields use typed builders, not function signatures with 15 `Option<_>` arguments.
5. **Explicit errors** — one `Error` enum; no `Box<dyn std::error::Error>`.
6. **No hidden global state** — `SchwabClient` and `StreamClient` are plain structs; callers own them.

---

## Repository Layout

```
schwab_api/
├── Cargo.toml
├── CLAUDE.md               ← this file
└── src/
    ├── lib.rs              ← public re-exports
    ├── error.rs            ← unified Error type
    ├── auth.rs             ← OAuth2 token management
    ├── client.rs           ← REST HTTP client (SchwabClient)
    ├── stream/
    │   ├── mod.rs          ← StreamClient, connect(), recv_loop
    │   ├── protocol.rs     ← WebSocket wire format (login/subs JSON)
    │   ├── services.rs     ← per-service sub/add_symbols/unsubs helpers
    │   └── fields.rs       ← all field enums (typed numeric→name mapping)
    └── models/
        ├── mod.rs
        ├── account.rs
        ├── quotes.rs
        ├── orders.rs
        ├── price_history.rs
        ├── options.rs
        ├── instruments.rs
        ├── transactions.rs
        ├── market_hours.rs
        ├── movers.rs
        └── stream/
            ├── mod.rs
            ├── level_one.rs        ← LevelOneEquity, LevelOneFutures, etc.
            ├── chart.rs            ← ChartEquity, ChartFutures
            ├── book.rs             ← NyseBook, NasdaqBook, OptionsBook
            ├── screener.rs
            └── account_activity.rs
```

---

## Dependencies

```toml
[dependencies]
tokio             = { version = "1", features = ["full"] }
reqwest           = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }
futures-util      = "0.3"
serde             = { version = "1", features = ["derive"] }
serde_json        = "1"
thiserror         = "2"
url               = "2"
chrono            = { version = "0.4", features = ["serde"] }
tracing           = "0.1"
tracing-subscriber = "0.3"
rustls            = { version = "0.23", default-features = false, features = ["ring", "std"] }
rustls-pemfile    = "2"
tokio-rustls      = "0.26"
```

---

## Authentication (`auth.rs`)

### Public API — one call

```rust
let tokens = TokenManager::create(config, Path::new("tokens.json")).await?;
```

- If `tokens.json` exists and refresh token is valid → loaded silently.
- If file missing or refresh token expired → prints authorize URL to stdout, starts local
  HTTPS server, waits for Schwab's redirect, extracts `code`, exchanges for tokens, saves file.

### `OAuthConfig`

```rust
pub struct OAuthConfig {
    pub app_key:       String,
    pub app_secret:    String,
    pub redirect_uri:  String,    // e.g. "https://127.0.0.1:8443"
    pub tls_cert_path: PathBuf,   // PEM cert for the local callback server
    pub tls_key_path:  PathBuf,   // PEM key for the local callback server
}
```

### `TokenManager` — opaque

No public methods. `get_valid_token()` is `pub(crate)`, used by `SchwabClient` and `StreamClient`.
Auto-refreshes access token when within 60s of expiry. Persists refreshed tokens to file
asynchronously (fire-and-forget `tokio::spawn`).

### OAuth callback server

On first run:
1. Loads TLS cert/key from `tls_cert_path` / `tls_key_path` (user-provided PEM files)
2. Binds `TcpListener` on the host:port from `redirect_uri`
3. Prints authorize URL
4. Accepts one HTTPS connection (Schwab's redirect)
5. Parses `code` from the request query string
6. Sends HTML success page
7. Exchanges code for tokens, saves to file

---

## REST Client (`client.rs`)

### `SchwabClient`

```rust
pub struct SchwabClient {
    http:   reqwest::Client,
    tokens: Arc<TokenManager>,
}
```

All methods `async`, return `Result<T>`. `Authorization: Bearer` injected automatically.

### Method Catalog

#### Accounts
- `get_account_numbers() -> Vec<AccountNumber>`
- `get_account(hash, fields) -> Account`
- `get_accounts(fields) -> Vec<Account>`
- `get_user_preferences() -> UserPreferences`

#### Quotes
- `get_quote(symbol, fields) -> QuoteResponse`
- `get_quotes(symbols, fields, indicative) -> QuotesMap`

#### Price History
- `get_price_history(req) -> PriceHistory`
- Convenience wrappers: `get_price_history_every_minute`, `_five_minutes`, `_day`, `_week`, etc.

#### Options
- `get_option_chain(req) -> OptionChain`

#### Instruments
- `get_instruments(symbols, projection) -> Vec<Instrument>`
- `get_instrument_by_cusip(cusip) -> Instrument`

#### Orders
- `place_order(hash, order) -> OrderId`
- `get_order(hash, order_id) -> Order`
- `get_orders_for_account(hash, req) -> Vec<Order>`
- `get_orders_for_all_accounts(req) -> Vec<Order>`
- `cancel_order(hash, order_id) -> ()`
- `replace_order(hash, order_id, order) -> ()`

#### Transactions
- `get_transaction(hash, tx_id) -> Transaction`
- `get_transactions(hash, req) -> Vec<Transaction>`

#### Market Data
- `get_movers(index, sort, frequency) -> Vec<Mover>`
- `get_market_hours(markets, date) -> MarketHours`

---

## Streaming (`stream/`)

### Architecture

```
                ┌──────────────────────────────────────────────┐
                │  StreamClient  (Arc<StreamClientInner>)      │
                │                                              │
                │  ws_sink ──► WebSocket ──► Schwab server    │
                │  ws_stream ◄──────────────────────────────  │
                │       │                                      │
                │  recv_loop task (spawned on connect)         │
                │       │                                      │
                │       ├──► response_tx (oneshot)  ← req/res │
                │       └──► service dispatcher                │
                │               │                             │
                │               ├──► mpsc::Sender<LevelOneEquityEvent>  │
                │               ├──► mpsc::Sender<ChartEquityEvent>     │
                │               └──► ... per subscribed service         │
                └──────────────────────────────────────────────┘
```

### Token Refresh and the Streaming Session

**The access token is only used once** — in the initial `ADMIN/LOGIN` message. Schwab's streaming
server does not invalidate an active WebSocket session when the 30-minute access token expires.
No mid-session token refresh is needed.

`Arc<TokenManager>` is held solely for **reconnection** after a dropped connection.

### Automatic Transparent Reconnect

Connection drops are fully handled inside `recv_loop`. Callers never see them — their
`mpsc::Receiver<*Event>` handles remain valid and resume after reconnect.
A `tracing::warn!` is emitted during the gap.

**Backoff schedule:**

| Attempt | Delay |
|---------|-------|
| 1 | 1 s |
| 2 | 2 s |
| 3 | 4 s |
| 4 | 8 s |
| 5 | 16 s |
| 6+ | 30 s (cap) |

Jitter (0–399 ms, from subsecond system time) applied to each delay.
Backoff resets on successful reconnect.
Loop exits early if all `mpsc::Sender` handles are closed (no consumers left).

On reconnect, all `active_subs` are replayed with `SUBS` (server has no prior state).

### `StreamClient`

```rust
pub struct StreamClient { /* Arc<StreamClientInner> + Mutex<Option<JoinHandle>> */ }

impl StreamClient {
    /// Returns Arc<StreamClient>. Connection torn down when last Arc is dropped.
    pub async fn connect(tokens: Arc<TokenManager>, preferences: UserPreferences) -> Result<Arc<Self>>;

    /// Explicit graceful logout with async wait. Optional — Drop handles it too.
    pub async fn logout(&self) -> Result<()>;
}

impl Drop for StreamClient {
    // Sends shutdown signal + aborts background task.
}
```

### Subscription API

```rust
// First call — creates channel, returns Receiver
let rx = stream.level_one_equities()
    .subscribe(&["AAPL"], &fields, capacity).await?;

// Expand — no new Receiver
stream.level_one_equities()
    .add_symbols(&["MSFT"], &fields).await?;

// Shrink
stream.level_one_equities()
    .unsubscribe(&["AAPL"]).await?;
```

All 13 services: `level_one_equities`, `level_one_options`, `level_one_futures`,
`level_one_forex`, `level_one_futures_options`, `chart_equity`, `chart_futures`,
`nyse_book`, `nasdaq_book`, `options_book`, `screener_equity`, `screener_option`,
`account_activity`.

### Streaming Event Structs

All fields except `symbol` are `Option<T>` — Schwab sends partial updates only.
Callers maintaining a full state snapshot must merge incoming events themselves.
Millisecond timestamp fields are converted to `DateTime<Utc>`.

---

## Error Handling (`error.rs`)

```rust
pub enum Error {
    Http(reqwest::Error),
    WebSocket(tungstenite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    OAuth { code, message },
    StreamLoginFailed { code, msg },
    UnexpectedStreamResponse { requestid },
    SubscriptionFailed { code, msg },
    AlreadySubscribed { service },
    NotSubscribed { service },
    StreamDisconnected,
    TokenExpired,
    Api { status, body },
}
```

---

## Concurrency Model for Streaming

- **Send serialization**: `Mutex<()>` guards all outbound commands — prevents interleaved request/response frames.
- **Response correlation**: `Mutex<Option<oneshot::Sender>>` — recv_loop delivers next `response` frame and clears the slot.
- **Data fan-out**: `HashMap<service, mpsc::Sender<Value>>` — typed converter task per service bridges raw JSON to typed events.
- **Back-pressure**: `send().await` blocks recv_loop when channel full — throttles all the way to TCP receive buffer, non-lossy.

---

## Key Implementation Notes

### Schwab API Base URLs
- REST: `https://api.schwabapi.com/marketdata/v1/` (market data)
- REST: `https://api.schwabapi.com/trader/v1/` (trading/accounts)
- OAuth: `https://api.schwabapi.com/v1/oauth/`
- Streaming: dynamic — from `GET /trader/v1/userPreference` → `streamerInfo[0].streamerSocketUrl`

### WebSocket Login Credentials
From `GET /trader/v1/userPreference` response `streamerInfo[0]`:
`streamerSocketUrl`, `schwabClientCorrelId`, `schwabClientCustomerId`,
`schwabClientChannel`, `schwabClientFunctionId`.

### Numeric Field Keys
Schwab streaming sends `{"0": "AAPL", "1": 150.25}` — string-numeric keys, partial updates only.

### Account Hash
Most endpoints require the account **hash** (not number). Obtained from `get_account_numbers()`.

---

## Example Usage

```rust
let config = OAuthConfig {
    app_key:       "...".to_string(),
    app_secret:    "...".to_string(),
    redirect_uri:  "https://127.0.0.1:8443".to_string(),
    tls_cert_path: "cert.pem".into(),
    tls_key_path:  "key.pem".into(),
};

// First run: prints URL, waits for redirect, saves tokens.json
// Subsequent runs: loads tokens.json silently
let tokens = TokenManager::create(config, Path::new("tokens.json")).await?;

let client = SchwabClient::new(Arc::clone(&tokens));
let quotes = client.get_quotes(&["AAPL"], None, None).await?;

let prefs = client.get_user_preferences().await?;
let stream = StreamClient::connect(Arc::clone(&tokens), prefs).await?;

let mut rx = stream
    .level_one_equities()
    .subscribe(&["AAPL", "MSFT"], &LevelOneEquityField::all(), 256)
    .await?;

while let Some(event) = rx.recv().await {
    println!("{}: bid={:?} ask={:?}", event.symbol, event.bid_price, event.ask_price);
}
```
