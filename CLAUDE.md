# SchwabApi — Rust Client Implementation Plan

## Project Overview

An idiomatic, async-first Rust client for the Charles Schwab brokerage API, based on analysis
of the `schwab-py` reference implementation. Streaming is the primary use case.

---

## Design Principles

1. **Typed everything** — every request parameter and response field is a named Rust type (struct/enum). No generic `Value` or `HashMap<String, serde_json::Value>` in the public API.
2. **Async-first** — tokio throughout; `reqwest` for HTTP, `tokio-tungstenite` for WebSocket.
3. **Streaming via bounded mpsc** — `tokio::sync::mpsc::channel` with a caller-specified capacity. No broadcast (lossy). Back-pressure propagates naturally.
4. **Builder pattern for requests** — optional fields use typed builders, not function signatures with 15 `Option<_>` arguments.
5. **Explicit errors** — one `Error` enum per module; no `Box<dyn std::error::Error>`.
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
    │   ├── mod.rs          ← StreamClient, connect()
    │   ├── protocol.rs     ← WebSocket wire format (login/subs JSON)
    │   ├── services.rs     ← per-service sub/unsub/add helpers
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

## Dependencies (Cargo.toml plan)

```toml
[dependencies]
tokio          = { version = "1", features = ["full"] }
reqwest        = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
thiserror      = "2"
url            = "2"
chrono         = { version = "0.4", features = ["serde"] }
tracing        = "0.1"

[dev-dependencies]
tokio          = { version = "1", features = ["full", "test-util"] }
```

---

## Authentication (`auth.rs`)

### Flow
Charles Schwab uses OAuth 2.0 Authorization Code flow:

1. User navigates to `https://api.schwabapi.com/v1/oauth/authorize?response_type=code&client_id=<APP_KEY>&redirect_uri=<REDIRECT_URI>`
2. After login, Schwab redirects to `https://127.0.0.1` with `?code=<AUTH_CODE>`
3. Exchange auth code for tokens: `POST https://api.schwabapi.com/v1/oauth/token`
4. Receive `access_token` (30 min TTL) + `refresh_token` (~7 day TTL)
5. Auto-refresh access token using refresh token before expiry

### Structs

```rust
pub struct TokenSet {
    pub access_token:  String,
    pub refresh_token: String,
    pub expires_at:    DateTime<Utc>,   // access token expiry
    pub refresh_expires_at: DateTime<Utc>,
}

pub struct OAuthConfig {
    pub app_key:      String,
    pub app_secret:   String,
    pub redirect_uri: String,
}
```

### `TokenManager`
- Holds `Arc<RwLock<TokenSet>>`
- Exposes `async fn get_valid_token() -> Result<String>` — refreshes automatically if within 60s of expiry
- Exposes `fn authorize_url() -> Url` — generates the user-facing login URL
- Exposes `async fn exchange_code(code: &str) -> Result<TokenSet>`
- Exposes `async fn save_token(path: &Path)` / `fn load_token(path: &Path) -> Result<TokenSet>` — JSON file persistence

---

## REST Client (`client.rs`)

### `SchwabClient`

```rust
pub struct SchwabClient {
    http:    reqwest::Client,
    tokens:  Arc<TokenManager>,
    base_url: Url,  // https://api.schwabapi.com/trader/v1/
}
```

All methods are `async` and return `Result<T, Error>`.
The client injects `Authorization: Bearer <token>` automatically via a middleware wrapper around `reqwest`.

### Method Catalog

#### Accounts
- `get_account_numbers() -> Vec<AccountNumber>`
- `get_account(account_hash: &str, fields: Option<AccountFields>) -> Account`
- `get_accounts(fields: Option<AccountFields>) -> Vec<Account>`
- `get_user_preferences() -> UserPreferences`  ← also used by StreamClient

#### Quotes
- `get_quote(symbol: &str, fields: Option<QuoteFields>) -> Quote`
- `get_quotes(symbols: &[&str], fields: Option<QuoteFields>, indicative: Option<bool>) -> HashMap<String, Quote>`

#### Price History
- `get_price_history(req: PriceHistoryRequest) -> PriceHistory`
- Convenience wrappers: `get_price_history_every_minute`, `_five_minutes`, `_day`, `_week`, etc.

#### Options
- `get_option_chain(req: OptionChainRequest) -> OptionChain`

#### Instruments
- `get_instruments(symbols: &[&str], projection: Projection) -> Vec<Instrument>`
- `get_instrument_by_cusip(cusip: &str) -> Instrument`

#### Orders
- `place_order(account_hash: &str, order: &Order) -> OrderId`
- `get_order(account_hash: &str, order_id: OrderId) -> Order`
- `get_orders_for_account(account_hash: &str, req: GetOrdersRequest) -> Vec<Order>`
- `get_orders_for_all_accounts(req: GetOrdersRequest) -> Vec<Order>`
- `cancel_order(account_hash: &str, order_id: OrderId) -> ()`
- `replace_order(account_hash: &str, order_id: OrderId, order: &Order) -> ()`

#### Transactions
- `get_transaction(account_hash: &str, tx_id: TransactionId) -> Transaction`
- `get_transactions(account_hash: &str, req: GetTransactionsRequest) -> Vec<Transaction>`

#### Market Data
- `get_movers(index: Index, sort: SortOrder, frequency: MoverFrequency) -> Vec<Mover>`
- `get_market_hours(markets: &[Market], date: NaiveDate) -> MarketHours`

---

## Streaming (`stream/`)

### Architecture

```
                ┌──────────────────────────────────────────────┐
                │  StreamClient                                │
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
server does not invalidate an active WebSocket session when the 30-minute access token expires;
the session is already authenticated. No mid-session token refresh is needed.

The `Arc<TokenManager>` is held solely for **reconnection** after a dropped connection.

### Automatic Transparent Reconnect

The connection drop is fully handled inside `recv_loop`. Callers never see it — their
`mpsc::Receiver<*Event>` handles remain valid and simply resume receiving data after reconnect.
During the gap, data is missed silently; a `tracing::warn!` is emitted.

**Reconnect loop structure** (inside `recv_loop` task):

```
loop {
    match session(inner).await {
        SessionExit::Clean =>
            break,                          // logout() was called — intentional stop

        SessionExit::Error(e) => {
            tracing::warn!("stream disconnected: {e}");
            backoff.wait().await;           // exponential sleep
            backoff.step();                 // double the delay, cap at MAX_BACKOFF
            // fall through → retry session()
        }
    }
}
// Only reached on clean exit or all mpsc receivers dropped.
// Close all senders so any lingering receivers get None.
```

**`session(inner)`** does the per-connection lifecycle:
1. `TokenManager::get_valid_token()` — refreshes if needed
2. Dial WSS (`tokio_tungstenite::connect_async`)
3. Send `ADMIN/LOGIN`, await response
4. Replay all entries in `inner.active_subs` (SUBS for each service)
5. Run the read loop until a WebSocket error or close frame
6. Return `SessionExit::Error(...)` so the outer loop can retry

**Backoff schedule** (no external crate needed):

| Attempt | Delay |
|---------|-------|
| 1 | 1 s |
| 2 | 2 s |
| 3 | 4 s |
| 4 | 8 s |
| 5 | 16 s |
| 6+ | 30 s (cap) |

Jitter (±20%) added to each delay to avoid thundering-herd if many clients reconnect at once.

On **successful reconnect** the backoff resets to 1 s.

**Early exit condition**: if all `mpsc::Sender` handles report every receiver is dropped
(i.e., `sender.is_closed()` for every service), there is no consumer left — the loop exits
cleanly rather than reconnecting into the void.

### Active Subscription Tracking

`StreamClientInner` keeps:

```rust
struct ActiveSub {
    service: &'static str,
    keys:    Vec<String>,        // symbols
    fields:  Vec<u32>,           // numeric field IDs
}

// inside StreamClientInner:
active_subs: Mutex<Vec<ActiveSub>>
```

`subscribe()` inserts a new entry (errors if one already exists for that service).
`add_symbols()` merges symbols+fields into the existing entry.
`unsubscribe()` prunes symbols; removes the entry entirely if none remain.
On reconnect, all `active_subs` are replayed using `SUBS` (not `ADD`) since the server has no
prior state after a reconnect.

### `StreamClient`

```rust
pub struct StreamClient {
    // internal: Arc<StreamClientInner>
}

impl StreamClient {
    /// Connect, authenticate, and start the background recv+reconnect loop.
    pub async fn connect(tokens: Arc<TokenManager>, preferences: UserPreferences) -> Result<Self>;

    /// Signal clean logout; background task exits after the current session closes.
    pub async fn logout(self) -> Result<()>;
}
```

The `recv_loop` is a `tokio::task::spawn`-ed async task that:
1. Outer loop: reconnect with exponential backoff (see above)
2. Inner session: reads raw WebSocket frames, parses JSON
3. Routes `response` frames to pending `oneshot::Sender`
4. Routes `data` frames by `service` name to the matching `mpsc::Sender`
5. Routes `notify/heartbeat` — `tracing::trace!` only, no response
6. On WebSocket error/close → exits inner session, outer loop retries

### Subscription API Pattern

Each service exposes three methods. Example for Level One Equities:

```rust
pub struct LevelOneEquitySub {
    client: Arc<StreamClientInner>,
}

impl LevelOneEquitySub {
    /// Initial subscription. Sends SUBS, creates the bounded channel, returns the Receiver.
    /// Errors if already subscribed (call add_symbols instead).
    pub async fn subscribe(
        &self,
        symbols:  &[&str],
        fields:   &[LevelOneEquityField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneEquityEvent>>;

    /// Add more symbols to an existing subscription. Sends ADD for symbols not already
    /// subscribed. Must call subscribe() first.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[LevelOneEquityField]) -> Result<()>;

    /// Remove symbols. If all symbols are removed, sends UNSUBS and closes the channel.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()>;
}
```

`StreamClient` exposes accessors for each service:
```rust
impl StreamClient {
    pub fn level_one_equities(&self)       -> LevelOneEquitySub;
    pub fn level_one_options(&self)        -> LevelOneOptionSub;
    pub fn level_one_futures(&self)        -> LevelOneFuturesSub;
    pub fn level_one_forex(&self)          -> LevelOneForexSub;
    pub fn level_one_futures_options(&self)-> LevelOneFuturesOptionsSub;
    pub fn chart_equity(&self)             -> ChartEquitySub;
    pub fn chart_futures(&self)            -> ChartFuturesSub;
    pub fn nyse_book(&self)                -> NyseBookSub;
    pub fn nasdaq_book(&self)              -> NasdaqBookSub;
    pub fn options_book(&self)             -> OptionsBookSub;
    pub fn screener_equity(&self)          -> ScreenerEquitySub;
    pub fn screener_option(&self)          -> ScreenerOptionSub;
    pub fn account_activity(&self)         -> AccountActivitySub;
}
```

### Wire Protocol (`stream/protocol.rs`)

Internal structs for JSON serialization only (not public).

**Outgoing:**
```rust
// Envelope
struct WireRequest<'a> { requests: Vec<WireRequestItem<'a>> }
struct WireRequestItem<'a> {
    service:                    &'a str,
    requestid:                  String,
    command:                    &'a str,
    #[serde(rename = "SchwabClientCustomerId")]
    schwab_client_customer_id:  &'a str,
    #[serde(rename = "SchwabClientCorrelId")]
    schwab_client_correl_id:    &'a str,
    parameters:                 serde_json::Value,
}
```

**Incoming:**
```rust
struct WireIncoming {
    response: Option<Vec<WireResponse>>,
    data:     Option<Vec<WireData>>,
    notify:   Option<Vec<WireNotify>>,
}
struct WireResponse { requestid: String, service: String, command: String, content: WireResponseContent }
struct WireResponseContent { code: i32, msg: String }
struct WireData { service: String, command: String, timestamp: Option<i64>, content: Vec<serde_json::Value> }
struct WireNotify { heartbeat: Option<i64> }
```

### Field Enums & Event Types (`stream/fields.rs`, `models/stream/`)

Each service has:
- A `*Field` enum listing all subscribable fields (numeric value via `repr(u32)` or `as u32`)
- An `*Event` struct with `Option<T>` fields for every possible field (because Schwab sends partial updates)

Example:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneEquityField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    // ... all 52 fields
}

#[derive(Debug, Clone, Default)]
pub struct LevelOneEquityEvent {
    pub symbol:      String,   // field 0 — always present
    pub bid_price:   Option<f64>,
    pub ask_price:   Option<f64>,
    pub last_price:  Option<f64>,
    pub bid_size:    Option<u64>,
    pub ask_size:    Option<u64>,
    // ... all other fields as Option<T>
    pub quote_time:  Option<DateTime<Utc>>,   // millis fields converted
    pub trade_time:  Option<DateTime<Utc>>,
}
```

The `*Event` deserialization maps numeric string keys (`"1"`, `"2"`) to struct fields using a custom `Deserialize` impl or a parse helper — **not** via serde attribute magic on the wire type, keeping protocol and model layers separate.

Full field lists implemented:
- `LevelOneEquityField` (52 fields)
- `LevelOneOptionField` (56 fields)
- `LevelOneFuturesField` (41 fields)
- `LevelOneForexField` (30 fields)
- `LevelOneFuturesOptionField` (32 fields)
- `ChartEquityField` (9 fields)
- `ChartFuturesField` (7 fields)
- `BookField` / `BidField` / `AskField` (for Level 2 books)
- `ScreenerField` (5 fields)
- `AccountActivityField` (4 fields)

---

## Error Handling (`error.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OAuth error: {code} — {message}")]
    OAuth { code: String, message: String },

    #[error("Stream login failed: code={code}, msg={msg}")]
    StreamLoginFailed { code: i32, msg: String },

    #[error("Unexpected stream response for requestid={requestid}")]
    UnexpectedStreamResponse { requestid: String },

    #[error("Stream subscription failed: code={code}, msg={msg}")]
    SubscriptionFailed { code: i32, msg: String },

    #[error("Already subscribed to service '{service}' — use add_symbols() to expand")]
    AlreadySubscribed { service: &'static str },

    #[error("Not subscribed to service '{service}' — call subscribe() first")]
    NotSubscribed { service: &'static str },

    #[error("Stream disconnected")]
    StreamDisconnected,

    #[error("Token expired — re-authentication required")]
    TokenExpired,

    #[error("API error: status={status}, body={body}")]
    Api { status: u16, body: String },
}
```

---

## Concurrency Model for Streaming

### Request serialization
A single `tokio::sync::Mutex<()>` guards all outbound subscription commands (matching schwab-py's asyncio.Lock). This prevents interspersed request/response frames.

### Response correlation
Before sending any request, the caller stores a `oneshot::Sender<WireResponse>` in `Arc<Mutex<Option<oneshot::Sender<WireResponse>>>>`. The recv_loop delivers the next `response` frame to it and clears the slot.

### Data fan-out
The recv_loop holds a `HashMap<ServiceName, mpsc::Sender<serde_json::Value>>`. When `subscribe()` is called, it inserts a new sender. When the receiver is dropped, the next send returns `SendError` and the dispatcher removes that entry (auto-cleanup).

### Back-pressure
`mpsc::channel(capacity)` — capacity is caller-provided. The dispatcher always uses `send().await`,
which blocks the recv loop when the channel is full. This naturally throttles the WebSocket reader
and applies back-pressure all the way to the TCP receive buffer — the safe, non-lossy choice.

---

## Implementation Phases

### Phase 1 — Foundation
- [ ] `Cargo.toml` with all dependencies
- [ ] `error.rs`
- [ ] `auth.rs`: `TokenSet`, `OAuthConfig`, `TokenManager` with token refresh
- [ ] `client.rs`: `SchwabClient` struct + reqwest setup + auth middleware
- [ ] `models/account.rs`, `models/quotes.rs` (basic types to validate REST calls)
- [ ] `client.rs`: `get_account_numbers`, `get_quote`, `get_quotes`

### Phase 2 — Full REST API
- [ ] All remaining models (orders, price history, options, instruments, transactions, market hours, movers)
- [ ] All remaining client methods

### Phase 3 — Streaming Core
- [ ] `stream/protocol.rs`: wire types + serialization
- [ ] `stream/mod.rs`: `StreamClient::connect`, recv_loop task, request/response correlation
- [ ] `stream/fields.rs`: all field enums with numeric values
- [ ] Streaming connection test (login/logout)

### Phase 4 — Streaming Services
- [ ] `models/stream/level_one.rs`: all LevelOne event structs + numeric→field deserialization
- [ ] `stream/services.rs`: subscribe/add/unsubscribe for all 13 services
- [ ] `models/stream/chart.rs`, `book.rs`, `screener.rs`, `account_activity.rs`

### Phase 5 — Polish
- [ ] `tracing` instrumentation throughout
- [ ] Doc comments on all public items
- [ ] Integration tests (gated behind `--features integration` + env var for credentials)
- [ ] Example binaries in `examples/`

---

## Key Implementation Notes

### Schwab API Base URLs
- REST: `https://api.schwabapi.com/marketdata/v1/` (market data)
- REST: `https://api.schwabapi.com/trader/v1/` (trading/accounts)
- OAuth: `https://api.schwabapi.com/v1/oauth/`
- Streaming: obtained dynamically from `GET /trader/v1/userPreference` → `streamerInfo[0].streamerSocketUrl`

### WebSocket Login Credential Sources
All from `GET /trader/v1/userPreference` response `streamerInfo[0]`:
- `streamerSocketUrl` — WSS URL
- `schwabClientCorrelId` — sent in every request
- `schwabClientCustomerId` — sent in every request
- `schwabClientChannel` — sent in LOGIN parameters
- `schwabClientFunctionId` — sent in LOGIN parameters

The `Authorization` parameter in LOGIN is the current access token string.

### Numeric Field Keys
Schwab streaming sends updates as objects with **string-numeric keys**: `{"0": "AAPL", "1": 150.25}`.
The key `"0"` is always the symbol (used as the routing key). All other keys are optional.
Deserialization must use a `Visitor` that matches string keys `"0"`..`"N"` to struct fields.

### Partial Updates
Streaming data messages contain **only changed fields**. Event structs must use `Option<T>` for every non-symbol field. Callers who need a full state snapshot must maintain their own state map and merge incoming updates.

### Timestamp Fields
Fields ending in `_MILLIS` (e.g., `QUOTE_TIME_MILLIS`) are Unix millisecond timestamps.
Convert to `DateTime<Utc>` using `chrono::DateTime::from_timestamp_millis`.

### Rate Limits / Token Refresh
The Schwab access token expires in 30 minutes. `TokenManager::get_valid_token` refreshes if within 60 seconds of expiry. The reqwest client middleware calls this before every request.

### Account Hash vs Account Number
Schwab's API requires the **account hash** (not the raw account number) for most account-specific endpoints. `get_account_numbers()` returns the mapping.

---

## Example Usage (target API shape)

```rust
// Auth
let config = OAuthConfig { app_key: "...", app_secret: "...", redirect_uri: "https://127.0.0.1" };
let tokens = TokenManager::load_or_authorize(&config, Path::new("tokens.json")).await?;

// REST
let client = SchwabClient::new(Arc::clone(&tokens));
let accounts = client.get_account_numbers().await?;
let quote = client.get_quote("AAPL", None).await?;

// Streaming
let prefs = client.get_user_preferences().await?;
let stream = StreamClient::connect(Arc::clone(&tokens), prefs).await?;

let mut rx = stream
    .level_one_equities()
    .subscribe(&["AAPL", "MSFT", "NVDA"], &LevelOneEquityField::all(), 256)
    .await?;

tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        println!("{}: bid={:?} ask={:?}", event.symbol, event.bid_price, event.ask_price);
    }
});
```
