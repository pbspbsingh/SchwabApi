//! Schwab Streaming API client.
//!
//! # Architecture
//!
//! [`StreamClient`] wraps a shared [`StreamClientInner`] behind an `Arc`.
//! On [`StreamClient::connect`] a background `recv_loop` actor task is spawned.
//! All mutable connection state lives exclusively inside that task — no `Mutex`
//! is needed for streaming state. User tasks communicate with the actor via a
//! bounded [`ActorCommand`] channel.
//!
//! The actor loop:
//! 1. Connects to the WSS endpoint and logs in.
//! 2. Replays any active subscriptions on reconnect.
//! 3. Accepts [`ActorCommand`]s (subscribe / add / unsub) only while idle
//!    (no pending server response), sends the wire request, then parks the
//!    command until the matching `response` frame arrives.
//! 4. Dispatches `data` frames to the per-service `mpsc::Sender`.
//! 5. On disconnect, retries with exponential back-off.

pub mod fields;
mod protocol;
pub mod services;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::auth::TokenManager;
use crate::error::{Error, Result};
use crate::models::account::UserPreferences;

use protocol::{WireIncoming, WireRequest, WireRequestItem, WireResponse};

// ── WebSocket type aliases ────────────────────────────────────────────────────

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;
type WsStreamSplit = futures_util::stream::SplitStream<WsStream>;

// ── Actor command ─────────────────────────────────────────────────────────────

/// Commands sent by user tasks to the `recv_loop` actor.
enum ActorCommand {
    Subscribe {
        service:  &'static str,
        keys:     Vec<String>,
        fields:   Vec<u32>,
        raw_tx:   mpsc::Sender<serde_json::Value>,
        reply:    oneshot::Sender<Result<()>>,
    },
    AddSymbols {
        service:  &'static str,
        keys:     Vec<String>,
        fields:   Vec<u32>,
        reply:    oneshot::Sender<Result<()>>,
    },
    Unsubscribe {
        service:  &'static str,
        keys:     Vec<String>,
        reply:    oneshot::Sender<Result<()>>,
    },
}

// ── Active subscription record ────────────────────────────────────────────────

/// Internal subscription state used for reconnect replay.
#[derive(Clone)]
struct ActiveSub {
    service: &'static str,
    keys:    Vec<String>,
    fields:  Vec<u32>,
}

// ── Actor state ───────────────────────────────────────────────────────────────

/// All mutable streaming state — owned exclusively by the `recv_loop` task.
struct ActorState {
    senders:    HashMap<String, mpsc::Sender<serde_json::Value>>,
    active_subs: Vec<ActiveSub>,
    /// A command whose wire request has been sent but whose response is pending.
    pending:    Option<ActorCommand>,
    request_id: u64,
}

// ── StreamClientInner (no Mutexes) ───────────────────────────────────────────

struct StreamClientInner {
    wss_url:     String,
    customer_id: String,
    correl_id:   String,
    channel:     String,
    function_id: String,
    tokens:      Arc<TokenManager>,
    /// Channel to the actor task.
    cmd_tx:      mpsc::Sender<ActorCommand>,
    /// Watch channel: send `true` to shut down the recv_loop.
    shutdown:    watch::Sender<bool>,
}

// ── Wire helpers ──────────────────────────────────────────────────────────────

/// Serialise and send one WebSocket request frame.
async fn wire_send(
    sink:       &mut WsSink,
    request_id: &mut u64,
    service:    &str,
    command:    &str,
    parameters: serde_json::Value,
    inner:      &StreamClientInner,
) -> Result<()> {
    let id = *request_id;
    *request_id += 1;
    let req = WireRequest {
        requests: vec![WireRequestItem {
            service,
            requestid: id.to_string(),
            command,
            customer_id: &inner.customer_id,
            correl_id:   &inner.correl_id,
            parameters,
        }],
    };
    sink.send(Message::Text(serde_json::to_string(&req)?.into())).await?;
    Ok(())
}

/// Read WebSocket frames until a `response` frame arrives.
/// Data and heartbeat frames received during this wait are silently ignored.
/// Used for login and subscription replay before the main event loop starts.
async fn wait_rpc_response(stream: &mut WsStreamSplit) -> Result<WireResponse> {
    loop {
        match stream.next().await {
            Some(Ok(Message::Text(text))) => {
                let incoming: WireIncoming = serde_json::from_str(&text)?;
                if let Some(mut responses) = incoming.response
                    && let Some(r) = responses.drain(..).next()
                {
                    return Ok(r);
                }
                // data / heartbeat during init — skip
            }
            Some(Ok(Message::Close(_))) | None => return Err(Error::StreamDisconnected),
            Some(Err(e)) => return Err(Error::WebSocket(e)),
            Some(Ok(_)) => {} // ping / pong / binary
        }
    }
}

// ── Actor helpers ─────────────────────────────────────────────────────────────

/// Dispatch an inbound text frame: fire pending reply, forward data, log heartbeats.
async fn handle_text(text: &str, state: &mut ActorState) -> Result<()> {
    let incoming: WireIncoming = serde_json::from_str(text)?;

    if let Some(responses) = incoming.response {
        for resp in responses {
            if let Some(pending) = state.pending.take() {
                complete_pending(pending, resp, state);
            }
        }
    }

    if let Some(data_items) = incoming.data {
        for data in data_items {
            // Clone the Sender (cheap — Arc-backed) and drop the map lock
            // before awaiting send, so the map is never held across an await.
            if let Some(tx) = state.senders.get(&data.service).cloned() {
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

/// Validate a command, send the wire request, and park the command as pending.
async fn handle_command(
    cmd:   ActorCommand,
    state: &mut ActorState,
    sink:  &mut WsSink,
    inner: &StreamClientInner,
) -> Result<()> {
    match cmd {
        ActorCommand::Subscribe { service, keys, fields, raw_tx, reply } => {
            if state.active_subs.iter().any(|s| s.service == service) {
                let _ = reply.send(Err(Error::AlreadySubscribed { service }));
                return Ok(());
            }
            let params = serde_json::json!({
                "keys":   keys.join(","),
                "fields": fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
            });
            wire_send(sink, &mut state.request_id, service, "SUBS", params, inner).await?;
            state.pending = Some(ActorCommand::Subscribe { service, keys, fields, raw_tx, reply });
        }
        ActorCommand::AddSymbols { service, keys, fields, reply } => {
            if !state.active_subs.iter().any(|s| s.service == service) {
                let _ = reply.send(Err(Error::NotSubscribed { service }));
                return Ok(());
            }
            let params = serde_json::json!({
                "keys":   keys.join(","),
                "fields": fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
            });
            wire_send(sink, &mut state.request_id, service, "ADD", params, inner).await?;
            state.pending = Some(ActorCommand::AddSymbols { service, keys, fields, reply });
        }
        ActorCommand::Unsubscribe { service, keys, reply } => {
            if !state.active_subs.iter().any(|s| s.service == service) {
                let _ = reply.send(Err(Error::NotSubscribed { service }));
                return Ok(());
            }
            let params = serde_json::json!({ "keys": keys.join(",") });
            wire_send(sink, &mut state.request_id, service, "UNSUBS", params, inner).await?;
            state.pending = Some(ActorCommand::Unsubscribe { service, keys, reply });
        }
    }
    Ok(())
}

/// Apply a server response to the parked command: update actor state and fire the reply.
fn complete_pending(pending: ActorCommand, resp: WireResponse, state: &mut ActorState) {
    let code = resp.content.code;
    let msg  = resp.content.msg;
    match pending {
        ActorCommand::Subscribe { service, keys, fields, raw_tx, reply } => {
            if code != 0 {
                let _ = reply.send(Err(Error::SubscriptionFailed { code, msg }));
                return;
            }
            state.senders.insert(service.to_string(), raw_tx);
            state.active_subs.push(ActiveSub { service, keys, fields });
            let _ = reply.send(Ok(()));
        }
        ActorCommand::AddSymbols { service, keys, reply, .. } => {
            if code != 0 {
                let _ = reply.send(Err(Error::SubscriptionFailed { code, msg }));
                return;
            }
            if let Some(sub) = state.active_subs.iter_mut().find(|s| s.service == service) {
                for key in &keys {
                    if !sub.keys.contains(key) { sub.keys.push(key.clone()); }
                }
            }
            let _ = reply.send(Ok(()));
        }
        ActorCommand::Unsubscribe { service, keys, reply } => {
            if code != 0 {
                let _ = reply.send(Err(Error::SubscriptionFailed { code, msg }));
                return;
            }
            if let Some(sub) = state.active_subs.iter_mut().find(|s| s.service == service) {
                sub.keys.retain(|k| !keys.contains(k));
                if sub.keys.is_empty() {
                    state.active_subs.retain(|s| s.service != service);
                    state.senders.remove(service);
                }
            }
            let _ = reply.send(Ok(()));
        }
    }
}

/// Reply to a parked command with a disconnection error (called on reconnect).
fn fail_pending(pending: ActorCommand) {
    let _ = match pending {
        ActorCommand::Subscribe  { reply, .. } => reply.send(Err(Error::StreamDisconnected)),
        ActorCommand::AddSymbols { reply, .. } => reply.send(Err(Error::StreamDisconnected)),
        ActorCommand::Unsubscribe { reply, .. } => reply.send(Err(Error::StreamDisconnected)),
    };
}

// ── Session ───────────────────────────────────────────────────────────────────

/// Run one WebSocket session to completion.
/// Returns `Ok(())` on clean shutdown; `Err` on any connection/protocol error.
async fn run_session(
    inner:       &StreamClientInner,
    state:       &mut ActorState,
    cmd_rx:      &mut mpsc::Receiver<ActorCommand>,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<()> {
    // 1. Connect.
    let token = inner.tokens.get_valid_token().await?;
    tracing::debug!("connecting to {}", inner.wss_url);
    let (ws, _) = tokio_tungstenite::connect_async(&inner.wss_url).await?;
    let (mut sink, mut stream) = ws.split();
    tracing::debug!("WebSocket connected");

    // 2. Login.
    let login_params = serde_json::json!({
        "Authorization":            token,
        "SchwabClientChannel":      inner.channel,
        "SchwabClientFunctionId":   inner.function_id,
    });
    wire_send(&mut sink, &mut state.request_id, "ADMIN", "LOGIN", login_params, inner).await?;
    let login_resp = wait_rpc_response(&mut stream).await?;
    if login_resp.content.code != 0 {
        return Err(Error::StreamLoginFailed {
            code: login_resp.content.code,
            msg:  login_resp.content.msg,
        });
    }
    tracing::info!("stream login OK");

    // 3. Replay active subscriptions.
    for sub in state.active_subs.clone() {
        tracing::debug!("replaying subscription for {}", sub.service);
        let params = serde_json::json!({
            "keys":   sub.keys.join(","),
            "fields": sub.fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
        });
        wire_send(&mut sink, &mut state.request_id, sub.service, "SUBS", params, inner).await?;
        if let Err(e) = wait_rpc_response(&mut stream).await {
            tracing::warn!("replay of {} failed: {e}", sub.service);
        }
    }

    // 4. Main event loop with heartbeat watchdog.
    //    Schwab sends a heartbeat every ~10 s; if nothing arrives for 15 s the
    //    connection is assumed stuck — tear it down and let recv_loop retry.
    let watchdog = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(watchdog);

    loop {
        tokio::select! {
            msg = stream.next() => {
                watchdog.as_mut().reset(
                    tokio::time::Instant::now() + Duration::from_secs(15),
                );
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_text(&text, state).await?;
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::warn!("stream: received Close frame");
                        return Err(Error::StreamDisconnected);
                    }
                    Some(Ok(_)) => {} // ping / pong / binary — ignore
                    Some(Err(e)) => return Err(Error::WebSocket(e)),
                    None => return Err(Error::StreamDisconnected),
                }
            }

            cmd = cmd_rx.recv(), if state.pending.is_none() => {
                match cmd {
                    Some(cmd) => handle_command(cmd, state, &mut sink, inner).await?,
                    None => return Ok(()), // all cmd senders dropped
                }
            }

            _ = &mut watchdog => {
                tracing::warn!("stream: no message for 15 s, connection assumed stuck");
                return Err(Error::StreamDisconnected);
            }

            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("stream: shutdown requested, logging out");
                    let _ = wire_send(
                        &mut sink, &mut state.request_id,
                        "ADMIN", "LOGOUT",
                        serde_json::Value::Object(Default::default()),
                        inner,
                    ).await;
                    return Ok(());
                }
            }
        }
    }
}

// ── recv_loop ─────────────────────────────────────────────────────────────────

async fn recv_loop(
    inner:       Arc<StreamClientInner>,
    mut cmd_rx:  mpsc::Receiver<ActorCommand>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut state = ActorState {
        senders:     HashMap::new(),
        active_subs: Vec::new(),
        pending:     None,
        request_id:  0,
    };
    let mut backoff_secs: u64 = 1;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        match run_session(&inner, &mut state, &mut cmd_rx, &mut shutdown_rx).await {
            Ok(()) => {
                tracing::info!("stream session ended cleanly");
                break;
            }
            Err(e) => {
                tracing::warn!(
                    "stream session ended with error: {e}, reconnecting in {backoff_secs}s",
                );
                // Fail any in-flight command so the caller isn't left hanging.
                if let Some(pending) = state.pending.take() {
                    fail_pending(pending);
                }
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                let jitter_ms = (nanos % 400) as u64;
                tokio::time::sleep(Duration::from_millis(
                    backoff_secs * 1000 + jitter_ms,
                )).await;
                backoff_secs = (backoff_secs * 2).min(30);
            }
        }

        // Early exit if all data receivers have been dropped.
        if !state.senders.is_empty() && state.senders.values().all(|s| s.is_closed()) {
            tracing::info!("all stream receivers dropped, stopping reconnect loop");
            break;
        }
    }

    // Close every sender so lingering receivers get `None`.
    state.senders.clear();
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
    inner:     Arc<StreamClientInner>,
    recv_task: Mutex<Option<JoinHandle<()>>>,
}

impl Drop for StreamClient {
    fn drop(&mut self) {
        let _ = self.inner.shutdown.send(true);
        if let Ok(mut guard) = self.recv_task.try_lock()
            && let Some(handle) = guard.take()
        {
            handle.abort();
        }
    }
}

impl StreamClient {
    /// Connect to the Schwab streaming server, authenticate, and start the
    /// background recv+reconnect loop.
    pub async fn connect(
        tokens:      Arc<TokenManager>,
        preferences: UserPreferences,
    ) -> Result<Arc<Self>> {
        let info = preferences
            .streamer_info
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 0,
                body:   "UserPreferences contained no streamerInfo".to_string(),
            })?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (cmd_tx, cmd_rx) = mpsc::channel::<ActorCommand>(32);

        let inner = Arc::new(StreamClientInner {
            wss_url:     info.streamer_socket_url,
            customer_id: info.schwab_client_customer_id,
            correl_id:   info.schwab_client_correl_id,
            channel:     info.schwab_client_channel,
            function_id: info.schwab_client_function_id,
            tokens,
            cmd_tx,
            shutdown: shutdown_tx,
        });

        let recv_inner = Arc::clone(&inner);
        let recv_task = tokio::spawn(recv_loop(recv_inner, cmd_rx, shutdown_rx));

        Ok(Arc::new(Self {
            inner,
            recv_task: Mutex::new(Some(recv_task)),
        }))
    }

    /// Explicitly signal a graceful logout and wait for the background task to
    /// finish. Optional — dropping the last `Arc<StreamClient>` also closes the
    /// connection (without the async wait).
    pub async fn logout(&self) -> Result<()> {
        let _ = self.inner.shutdown.send(true);
        let handle = self.recv_task.lock().await.take();
        if let Some(h) = handle {
            let _ = h.await;
        }
        Ok(())
    }

    // ── service accessors ──────────────────────────────────────────────────

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
