//! Per-service subscription helpers.
//!
//! Each `*Sub` struct provides three methods:
//! - `subscribe` — initial subscription, returns `mpsc::Receiver<*Event>`
//! - `add_symbols` — extend an existing subscription
//! - `unsubscribe` — remove symbols (removes entry when all symbols gone)

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::error::{Error, Result};
use crate::models::stream::account_activity::AccountActivityEvent;
use crate::models::stream::book::BookEvent;
use crate::models::stream::chart::{ChartEquityEvent, ChartFuturesEvent};
use crate::models::stream::level_one::{
    LevelOneEquityEvent, LevelOneFuturesEvent, LevelOneFuturesOptionsEvent, LevelOneForexEvent,
    LevelOneOptionEvent,
};
use crate::models::stream::screener::ScreenerEvent;
use crate::stream::fields::{
    AccountActivityField, BookField, ChartEquityField, ChartFuturesField, LevelOneEquityField,
    LevelOneForexField, LevelOneFuturesField, LevelOneFuturesOptionField, LevelOneOptionField,
    ScreenerField,
};
use crate::stream::StreamClientInner;

// ── Active subscription record ────────────────────────────────────────────────

/// Internal subscription state used for reconnect replay.
#[derive(Clone)]
pub struct ActiveSub {
    pub service: &'static str,
    pub keys: Vec<String>,
    pub fields: Vec<u32>,
}

// ── Generic helper macros ─────────────────────────────────────────────────────

/// Check that a service is NOT yet subscribed (for subscribe()).
async fn ensure_not_subscribed(
    inner: &StreamClientInner,
    service: &'static str,
) -> Result<()> {
    let subs = inner.active_subs.lock().await;
    if subs.iter().any(|s| s.service == service) {
        return Err(Error::AlreadySubscribed { service });
    }
    Ok(())
}

/// Check that a service IS already subscribed (for add/unsub).
async fn ensure_subscribed(
    inner: &StreamClientInner,
    service: &'static str,
) -> Result<()> {
    let subs = inner.active_subs.lock().await;
    if !subs.iter().any(|s| s.service == service) {
        return Err(Error::NotSubscribed { service });
    }
    Ok(())
}

// ── LevelOneEquitySub ─────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_EQUITIES` service.
pub struct LevelOneEquitySub {
    inner: Arc<StreamClientInner>,
}

impl LevelOneEquitySub {
    const SERVICE: &'static str = "LEVELONE_EQUITIES";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Level One equity data.
    ///
    /// Returns a receiver that yields [`LevelOneEquityEvent`] updates.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[LevelOneEquityField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneEquityEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        // Send subscription command.
        self.inner
            .send_subs(Self::SERVICE, &keys, &field_ids)
            .await?;

        // Create raw → typed bridge.
        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<LevelOneEquityEvent>(capacity);

        self.inner
            .senders
            .lock()
            .await
            .insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub {
            service: Self::SERVICE,
            keys,
            fields: field_ids,
        });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = LevelOneEquityEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(
        &self,
        symbols: &[&str],
        fields: &[LevelOneEquityField],
    ) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner
            .send_add(Self::SERVICE, &keys, &field_ids)
            .await?;

        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys {
                if !sub.keys.contains(key) {
                    sub.keys.push(key.clone());
                }
            }
        }
        Ok(())
    }

    /// Remove symbols from the subscription; unsubscribes entirely when empty.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;

        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;

        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner
                    .senders
                    .lock()
                    .await
                    .remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── LevelOneOptionSub ─────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_OPTIONS` service.
pub struct LevelOneOptionSub {
    inner: Arc<StreamClientInner>,
}

impl LevelOneOptionSub {
    const SERVICE: &'static str = "LEVELONE_OPTIONS";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Level One options data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[LevelOneOptionField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneOptionEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<LevelOneOptionEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = LevelOneOptionEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[LevelOneOptionField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── LevelOneFuturesSub ────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FUTURES` service.
pub struct LevelOneFuturesSub {
    inner: Arc<StreamClientInner>,
}

impl LevelOneFuturesSub {
    const SERVICE: &'static str = "LEVELONE_FUTURES";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Level One futures data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[LevelOneFuturesField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneFuturesEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<LevelOneFuturesEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = LevelOneFuturesEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[LevelOneFuturesField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── LevelOneForexSub ──────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FOREX` service.
pub struct LevelOneForexSub {
    inner: Arc<StreamClientInner>,
}

impl LevelOneForexSub {
    const SERVICE: &'static str = "LEVELONE_FOREX";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Level One forex data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[LevelOneForexField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneForexEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<LevelOneForexEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = LevelOneForexEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[LevelOneForexField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── LevelOneFuturesOptionsSub ─────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FUTURES_OPTIONS` service.
pub struct LevelOneFuturesOptionsSub {
    inner: Arc<StreamClientInner>,
}

impl LevelOneFuturesOptionsSub {
    const SERVICE: &'static str = "LEVELONE_FUTURES_OPTIONS";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Level One futures options data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[LevelOneFuturesOptionField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<LevelOneFuturesOptionsEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<LevelOneFuturesOptionsEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = LevelOneFuturesOptionsEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[LevelOneFuturesOptionField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── ChartEquitySub ────────────────────────────────────────────────────────────

/// Subscription helper for the `CHART_EQUITY` service.
pub struct ChartEquitySub {
    inner: Arc<StreamClientInner>,
}

impl ChartEquitySub {
    const SERVICE: &'static str = "CHART_EQUITY";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Chart Equity (1-min OHLCV) data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[ChartEquityField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<ChartEquityEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<ChartEquityEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = ChartEquityEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[ChartEquityField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── ChartFuturesSub ───────────────────────────────────────────────────────────

/// Subscription helper for the `CHART_FUTURES` service.
pub struct ChartFuturesSub {
    inner: Arc<StreamClientInner>,
}

impl ChartFuturesSub {
    const SERVICE: &'static str = "CHART_FUTURES";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Chart Futures (1-min OHLCV) data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[ChartFuturesField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<ChartFuturesEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<ChartFuturesEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = ChartFuturesEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[ChartFuturesField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── NyseBookSub ───────────────────────────────────────────────────────────────

/// Subscription helper for the `NYSE_BOOK` service.
pub struct NyseBookSub {
    inner: Arc<StreamClientInner>,
}

impl NyseBookSub {
    const SERVICE: &'static str = "NYSE_BOOK";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to NYSE Level 2 book data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[BookField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<BookEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<BookEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = BookEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[BookField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── NasdaqBookSub ─────────────────────────────────────────────────────────────

/// Subscription helper for the `NASDAQ_BOOK` service.
pub struct NasdaqBookSub {
    inner: Arc<StreamClientInner>,
}

impl NasdaqBookSub {
    const SERVICE: &'static str = "NASDAQ_BOOK";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to NASDAQ Level 2 book data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[BookField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<BookEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<BookEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = BookEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[BookField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── OptionsBookSub ────────────────────────────────────────────────────────────

/// Subscription helper for the `OPTIONS_BOOK` service.
pub struct OptionsBookSub {
    inner: Arc<StreamClientInner>,
}

impl OptionsBookSub {
    const SERVICE: &'static str = "OPTIONS_BOOK";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to Options Level 2 book data.
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        fields: &[BookField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<BookEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<BookEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = BookEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add symbols to an existing subscription.
    pub async fn add_symbols(&self, symbols: &[&str], fields: &[BookField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove symbols from the subscription.
    pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── ScreenerEquitySub ─────────────────────────────────────────────────────────

/// Subscription helper for the `SCREENER_EQUITY` service.
pub struct ScreenerEquitySub {
    inner: Arc<StreamClientInner>,
}

impl ScreenerEquitySub {
    const SERVICE: &'static str = "SCREENER_EQUITY";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to the equity screener.
    ///
    /// `keys` are screener keys such as `"$DJI_PERCENT_CHANGE_UP_60"`.
    pub async fn subscribe(
        &self,
        keys_arr: &[&str],
        fields: &[ScreenerField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<ScreenerEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<ScreenerEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = ScreenerEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add keys to an existing subscription.
    pub async fn add_symbols(&self, keys_arr: &[&str], fields: &[ScreenerField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove keys from the subscription.
    pub async fn unsubscribe(&self, keys_arr: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── ScreenerOptionSub ─────────────────────────────────────────────────────────

/// Subscription helper for the `SCREENER_OPTION` service.
pub struct ScreenerOptionSub {
    inner: Arc<StreamClientInner>,
}

impl ScreenerOptionSub {
    const SERVICE: &'static str = "SCREENER_OPTION";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to the option screener.
    pub async fn subscribe(
        &self,
        keys_arr: &[&str],
        fields: &[ScreenerField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<ScreenerEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<ScreenerEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = ScreenerEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add keys to an existing subscription.
    pub async fn add_symbols(&self, keys_arr: &[&str], fields: &[ScreenerField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Remove keys from the subscription.
    pub async fn unsubscribe(&self, keys_arr: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}

// ── AccountActivitySub ────────────────────────────────────────────────────────

/// Subscription helper for the `ACCT_ACTIVITY` service.
pub struct AccountActivitySub {
    inner: Arc<StreamClientInner>,
}

impl AccountActivitySub {
    const SERVICE: &'static str = "ACCT_ACTIVITY";

    pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
        Self { inner }
    }

    /// Subscribe to account activity events.
    ///
    /// The `keys` parameter is typically `&["Account Activity"]`.
    pub async fn subscribe(
        &self,
        keys_arr: &[&str],
        fields: &[AccountActivityField],
        capacity: usize,
    ) -> Result<mpsc::Receiver<AccountActivityEvent>> {
        ensure_not_subscribed(&self.inner, Self::SERVICE).await?;

        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();

        self.inner.send_subs(Self::SERVICE, &keys, &field_ids).await?;

        let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
        let (typed_tx, typed_rx) = mpsc::channel::<AccountActivityEvent>(capacity);

        self.inner.senders.lock().await.insert(Self::SERVICE.to_string(), raw_tx);

        let mut subs = self.inner.active_subs.lock().await;
        subs.push(ActiveSub { service: Self::SERVICE, keys, fields: field_ids });
        drop(subs);

        tokio::spawn(async move {
            while let Some(raw) = raw_rx.recv().await {
                if let Ok(event) = AccountActivityEvent::try_from(&raw)
                    && typed_tx.send(event).await.is_err()
                {
                    break;
                }
            }
        });

        Ok(typed_rx)
    }

    /// Add keys to an existing subscription.
    pub async fn add_symbols(&self, keys_arr: &[&str], fields: &[AccountActivityField]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_add(Self::SERVICE, &keys, &field_ids).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            for key in &keys { if !sub.keys.contains(key) { sub.keys.push(key.clone()); } }
        }
        Ok(())
    }

    /// Unsubscribe from account activity.
    pub async fn unsubscribe(&self, keys_arr: &[&str]) -> Result<()> {
        ensure_subscribed(&self.inner, Self::SERVICE).await?;
        let keys: Vec<String> = keys_arr.iter().map(|s| s.to_string()).collect();
        self.inner.send_unsubs(Self::SERVICE, &keys).await?;
        let mut subs = self.inner.active_subs.lock().await;
        if let Some(sub) = subs.iter_mut().find(|s| s.service == Self::SERVICE) {
            sub.keys.retain(|k| !keys.contains(k));
            if sub.keys.is_empty() {
                subs.retain(|s| s.service != Self::SERVICE);
                self.inner.senders.lock().await.remove(Self::SERVICE);
            }
        }
        Ok(())
    }
}
