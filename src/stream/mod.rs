//! Schwab Streaming API client.
//!
//! # Architecture
//!
//! [`StreamClient`] wraps a shared [`StreamClientInner`] behind an `Arc`.
//! On [`StreamClient::connect`] a background `recv_loop` task is spawned that:
//!
//! 1. Connects to the WSS endpoint and logs in.
//! 2. Replays any active subscriptions on reconnect.
//! 3. Reads frames and routes `response` frames to a pending oneshot, and
//!    `data` frames to the per-service `mpsc::Sender`.
//! 4. On disconnect, retries with exponential back-off.
//!
//! Callers obtain typed [`mpsc::Receiver`][tokio::sync::mpsc::Receiver]
//! handles via the service accessor methods (e.g. [`StreamClient::level_one_equities`]).

pub mod fields;
pub(crate) mod protocol;
pub mod services;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;

use crate::auth::TokenManager;
use crate::error::{Error, Result};
use crate::models::account::UserPreferences;

use protocol::{WireIncoming, WireRequest, WireRequestItem, WireResponse};
use services::ActiveSub;

// Concrete WebSocket stream type.
type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;

// ── StreamClientInner ─────────────────────────────────────────────────────────

pub(crate) struct StreamClientInner {
    /// WSS endpoint URL.
    pub(crate) wss_url: String,
    /// Schwab client customer ID (from UserPreferences).
    pub(crate) customer_id: String,
    /// Schwab correl ID (from UserPreferences).
    pub(crate) correl_id: String,
    /// Schwab channel (from UserPreferences).
    pub(crate) channel: String,
    /// Schwab function ID (from UserPreferences).
    pub(crate) function_id: String,

    pub(crate) tokens: Arc<TokenManager>,

    /// Serializes all outbound commands so responses can be correlated.
    pub(crate) send_lock: Mutex<()>,

    /// The WebSocket sink — locked briefly to write a frame.
    pub(crate) ws_sink: Mutex<Option<WsSink>>,

    /// Pending oneshot for the next expected `response` frame.
    pub(crate) pending_response: Mutex<Option<oneshot::Sender<Result<WireResponse>>>>,

    /// Per-service raw data senders (keyed by service name string).
    pub(crate) senders: Mutex<HashMap<String, mpsc::Sender<serde_json::Value>>>,

    /// Active subscriptions — replayed on reconnect.
    pub(crate) active_subs: Mutex<Vec<ActiveSub>>,

    /// Monotonically-increasing request ID.
    pub(crate) request_id: AtomicU64,

    /// Watch channel: send `true` to shut down the recv_loop.
    pub(crate) shutdown: watch::Sender<bool>,
}

impl StreamClientInner {
    /// Build and send a request, then await the response frame.
    pub(crate) async fn send_request(
        &self,
        service: &str,
        command: &str,
        parameters: serde_json::Value,
    ) -> Result<WireResponse> {
        let _guard = self.send_lock.lock().await;

        let req_id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let req = WireRequest {
            requests: vec![WireRequestItem {
                service,
                requestid: req_id.to_string(),
                command,
                customer_id: &self.customer_id,
                correl_id: &self.correl_id,
                parameters,
            }],
        };

        let (tx, rx) = oneshot::channel();
        *self.pending_response.lock().await = Some(tx);

        let text = serde_json::to_string(&req)?;
        {
            let mut sink_guard = self.ws_sink.lock().await;
            if let Some(sink) = sink_guard.as_mut() {
                sink.send(Message::Text(text.into())).await?;
            } else {
                return Err(Error::StreamDisconnected);
            }
        }

        rx.await.map_err(|_| Error::StreamDisconnected)?
    }

    /// Send the ADMIN/LOGIN message.
    pub(crate) async fn send_login(&self, token: &str) -> Result<()> {
        let params = serde_json::json!({
            "Authorization": token,
            "SchwabClientChannel": self.channel,
            "SchwabClientFunctionId": self.function_id,
        });
        let resp = self.send_request("ADMIN", "LOGIN", params).await?;
        if resp.content.code != 0 {
            return Err(Error::StreamLoginFailed {
                code: resp.content.code,
                msg: resp.content.msg,
            });
        }
        tracing::info!("stream login OK");
        Ok(())
    }

    /// Send the ADMIN/LOGOUT message (best-effort).
    pub(crate) async fn send_logout(&self) -> Result<()> {
        let _ = self
            .send_request("ADMIN", "LOGOUT", serde_json::Value::Object(Default::default()))
            .await;
        Ok(())
    }

    /// Send a SUBS command for the given service.
    pub(crate) async fn send_subs(
        &self,
        service: &str,
        keys: &[String],
        fields: &[u32],
    ) -> Result<WireResponse> {
        let params = serde_json::json!({
            "keys": keys.join(","),
            "fields": fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
        });
        self.send_request(service, "SUBS", params).await
    }

    /// Send an ADD command for the given service.
    pub(crate) async fn send_add(
        &self,
        service: &str,
        keys: &[String],
        fields: &[u32],
    ) -> Result<WireResponse> {
        let params = serde_json::json!({
            "keys": keys.join(","),
            "fields": fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
        });
        self.send_request(service, "ADD", params).await
    }

    /// Send an UNSUBS command for the given service.
    pub(crate) async fn send_unsubs(
        &self,
        service: &str,
        keys: &[String],
    ) -> Result<WireResponse> {
        let params = serde_json::json!({
            "keys": keys.join(","),
        });
        self.send_request(service, "UNSUBS", params).await
    }

    /// Parse and dispatch an incoming text frame.
    pub(crate) async fn handle_message(&self, text: &str) -> Result<()> {
        let incoming: WireIncoming = serde_json::from_str(text)?;

        if let Some(responses) = incoming.response {
            for resp in responses {
                if let Some(tx) = self.pending_response.lock().await.take() {
                    if resp.content.code != 0 {
                        let _ = tx.send(Err(Error::SubscriptionFailed {
                            code: resp.content.code,
                            msg: resp.content.msg.clone(),
                        }));
                    } else {
                        let _ = tx.send(Ok(resp));
                    }
                }
            }
        }

        if let Some(data_items) = incoming.data {
            for data in data_items {
                let senders = self.senders.lock().await;
                if let Some(tx) = senders.get(&data.service) {
                    for item in data.content {
                        if tx.send(item).await.is_err() {
                            // Receiver dropped — cleaned up lazily.
                        }
                    }
                }
            }
        }

        if let Some(notifies) = incoming.notify {
            for n in notifies {
                if let Some(ts) = n.heartbeat {
                    tracing::trace!("stream heartbeat: {ts}");
                }
            }
        }

        Ok(())
    }
}

// ── session ───────────────────────────────────────────────────────────────────

/// Run a single WebSocket session to completion.
///
/// Returns `Ok(())` on clean shutdown (logout requested).
/// Returns `Err(_)` on any connection or protocol error.
async fn run_session(
    inner: &Arc<StreamClientInner>,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<()> {
    // 1. Get a fresh access token.
    let token = inner.tokens.get_valid_token().await?;

    // 2. Connect.
    tracing::debug!("connecting to {}", inner.wss_url);
    let (ws_stream, _) = tokio_tungstenite::connect_async(&inner.wss_url).await?;
    let (sink, mut stream) = ws_stream.split();
    *inner.ws_sink.lock().await = Some(sink);
    tracing::debug!("WebSocket connected");

    // 3. Login.
    inner.send_login(&token).await?;

    // 4. Replay active subscriptions.
    let subs = inner.active_subs.lock().await.clone();
    for sub in subs {
        tracing::debug!("replaying subscription for {}", sub.service);
        inner.send_subs(sub.service, &sub.keys, &sub.fields).await?;
    }

    // 5. Read loop with heartbeat watchdog.
    //    Schwab sends a heartbeat every ~10 s. If nothing arrives for 15 s
    //    the connection is considered stuck — drop it and let recv_loop retry.
    let watchdog = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(watchdog);

    loop {
        tokio::select! {
            msg = stream.next() => {
                // Any frame resets the watchdog.
                watchdog.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(15));

                match msg {
                    Some(Ok(Message::Text(text))) => {
                        inner.handle_message(&text).await?;
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::warn!("stream: received Close frame");
                        return Err(Error::StreamDisconnected);
                    }
                    Some(Ok(_)) => {
                        // Ping / Pong / Binary — ignore.
                    }
                    Some(Err(e)) => {
                        return Err(Error::WebSocket(e));
                    }
                    None => {
                        return Err(Error::StreamDisconnected);
                    }
                }
            }
            _ = &mut watchdog => {
                tracing::warn!("stream: no message received for 15 s, connection assumed stuck");
                return Err(Error::StreamDisconnected);
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("stream: shutdown requested, logging out");
                    let _ = inner.send_logout().await;
                    return Ok(());
                }
            }
        }
    }
}

// ── recv_loop ─────────────────────────────────────────────────────────────────

async fn recv_loop(inner: Arc<StreamClientInner>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut backoff_secs: u64 = 1;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        match run_session(&inner, &mut shutdown_rx).await {
            Ok(()) => {
                tracing::info!("stream session ended cleanly");
                break;
            }
            Err(e) => {
                tracing::warn!("stream session ended with error: {e}, reconnecting in {backoff_secs}s");

                // Sleep with jitter (no rand crate needed).
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                let jitter_ms = (nanos % 400) as u64; // 0–399 ms
                let sleep_ms = backoff_secs * 1000 + jitter_ms;
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                backoff_secs = (backoff_secs * 2).min(30);
            }
        }

        // Early exit if all receivers have been dropped.
        {
            let senders = inner.senders.lock().await;
            if !senders.is_empty() && senders.values().all(|s| s.is_closed()) {
                tracing::info!("all stream receivers dropped, stopping reconnect loop");
                break;
            }
        }
    }

    // Close every sender so lingering receivers get `None`.
    inner.senders.lock().await.clear();
    tracing::info!("recv_loop exited");
}

// ── StreamClient (public) ─────────────────────────────────────────────────────

/// Schwab streaming client.
///
/// Schwab permits only **one** active streaming connection per account.
/// [`StreamClient::connect`] therefore returns `Arc<StreamClient>` — clone
/// the `Arc` freely to share the client across tasks; the underlying connection
/// is torn down automatically when the **last** `Arc` is dropped.
///
/// For an explicit, graceful shutdown call [`StreamClient::logout`].
pub struct StreamClient {
    inner: Arc<StreamClientInner>,
    /// Handle to the background recv_loop task.
    /// Wrapped in `Mutex<Option<…>>` so `logout()` can take and await it
    /// while `Drop` aborts whatever remains.
    recv_task: Mutex<Option<JoinHandle<()>>>,
}

impl Drop for StreamClient {
    fn drop(&mut self) {
        // Signal the recv_loop to stop.
        let _ = self.inner.shutdown.send(true);
        // Abort the background task as a safety net (non-blocking).
        if let Ok(mut guard) = self.recv_task.try_lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
    }
}

impl StreamClient {
    /// Connect to the Schwab streaming server, authenticate, and start the
    /// background recv+reconnect loop.
    ///
    /// Returns an `Arc<StreamClient>`. The connection is kept alive as long as
    /// at least one `Arc` clone exists; it is closed when the last clone drops.
    pub async fn connect(
        tokens: Arc<TokenManager>,
        preferences: UserPreferences,
    ) -> Result<Arc<Self>> {
        let info = preferences
            .streamer_info
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 0,
                body: "UserPreferences contained no streamerInfo".to_string(),
            })?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let inner = Arc::new(StreamClientInner {
            wss_url: info.streamer_socket_url,
            customer_id: info.schwab_client_customer_id,
            correl_id: info.schwab_client_correl_id,
            channel: info.schwab_client_channel,
            function_id: info.schwab_client_function_id,
            tokens,
            send_lock: Mutex::new(()),
            ws_sink: Mutex::new(None),
            pending_response: Mutex::new(None),
            senders: Mutex::new(HashMap::new()),
            active_subs: Mutex::new(Vec::new()),
            request_id: AtomicU64::new(0),
            shutdown: shutdown_tx,
        });

        let recv_inner = Arc::clone(&inner);
        let recv_task = tokio::spawn(recv_loop(recv_inner, shutdown_rx));

        Ok(Arc::new(Self {
            inner,
            recv_task: Mutex::new(Some(recv_task)),
        }))
    }

    /// Explicitly signal a graceful logout and wait for the background task to
    /// finish. After this returns the connection is closed.
    ///
    /// This is optional — dropping the last `Arc<StreamClient>` achieves the
    /// same effect (without the async wait).
    pub async fn logout(&self) -> Result<()> {
        let _ = self.inner.shutdown.send(true);
        let handle = self.recv_task.lock().await.take();
        if let Some(h) = handle {
            let _ = h.await;
        }
        Ok(())
    }

    // ── service accessors ─────────────────────────────────────────────────

    /// Access the Level One Equities subscription service.
    pub fn level_one_equities(&self) -> services::LevelOneEquitySub {
        services::LevelOneEquitySub::new(Arc::clone(&self.inner))
    }

    /// Access the Level One Options subscription service.
    pub fn level_one_options(&self) -> services::LevelOneOptionSub {
        services::LevelOneOptionSub::new(Arc::clone(&self.inner))
    }

    /// Access the Level One Futures subscription service.
    pub fn level_one_futures(&self) -> services::LevelOneFuturesSub {
        services::LevelOneFuturesSub::new(Arc::clone(&self.inner))
    }

    /// Access the Level One Forex subscription service.
    pub fn level_one_forex(&self) -> services::LevelOneForexSub {
        services::LevelOneForexSub::new(Arc::clone(&self.inner))
    }

    /// Access the Level One Futures Options subscription service.
    pub fn level_one_futures_options(&self) -> services::LevelOneFuturesOptionsSub {
        services::LevelOneFuturesOptionsSub::new(Arc::clone(&self.inner))
    }

    /// Access the Chart Equity subscription service.
    pub fn chart_equity(&self) -> services::ChartEquitySub {
        services::ChartEquitySub::new(Arc::clone(&self.inner))
    }

    /// Access the Chart Futures subscription service.
    pub fn chart_futures(&self) -> services::ChartFuturesSub {
        services::ChartFuturesSub::new(Arc::clone(&self.inner))
    }

    /// Access the NYSE Book subscription service.
    pub fn nyse_book(&self) -> services::NyseBookSub {
        services::NyseBookSub::new(Arc::clone(&self.inner))
    }

    /// Access the NASDAQ Book subscription service.
    pub fn nasdaq_book(&self) -> services::NasdaqBookSub {
        services::NasdaqBookSub::new(Arc::clone(&self.inner))
    }

    /// Access the Options Book subscription service.
    pub fn options_book(&self) -> services::OptionsBookSub {
        services::OptionsBookSub::new(Arc::clone(&self.inner))
    }

    /// Access the Screener Equity subscription service.
    pub fn screener_equity(&self) -> services::ScreenerEquitySub {
        services::ScreenerEquitySub::new(Arc::clone(&self.inner))
    }

    /// Access the Screener Option subscription service.
    pub fn screener_option(&self) -> services::ScreenerOptionSub {
        services::ScreenerOptionSub::new(Arc::clone(&self.inner))
    }

    /// Access the Account Activity subscription service.
    pub fn account_activity(&self) -> services::AccountActivitySub {
        services::AccountActivitySub::new(Arc::clone(&self.inner))
    }
}
