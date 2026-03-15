//! Per-service subscription helpers.
//!
//! Each `*Sub` struct provides three methods:
//! - `subscribe` — initial subscription, returns `mpsc::Receiver<*Event>`
//! - `add_symbols` — extend an existing subscription
//! - `unsubscribe` — remove symbols (removes entry when all symbols gone)
//!
//! All three delegate to the `recv_loop` actor via [`ActorCommand`]; no locks
//! are held on this side.

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use crate::error::{Error, Result};
use crate::models::stream::account_activity::AccountActivityEvent;
use crate::models::stream::book::BookEvent;
use crate::models::stream::chart::{ChartEquityEvent, ChartFuturesEvent};
use crate::models::stream::level_one::{
    LevelOneEquityEvent, LevelOneForexEvent, LevelOneFuturesEvent,
    LevelOneFuturesOptionsEvent, LevelOneOptionEvent,
};
use crate::models::stream::screener::ScreenerEvent;
use crate::stream::fields::{
    AccountActivityField, BookField, ChartEquityField, ChartFuturesField, LevelOneEquityField,
    LevelOneForexField, LevelOneFuturesField, LevelOneFuturesOptionField, LevelOneOptionField,
    ScreenerField,
};
use crate::stream::{ActorCommand, StreamClientInner};

// ── Macro: generate the three subscription methods for a service ──────────────

macro_rules! impl_subscribe {
    // services with typed field enums
    (
        $Struct:ident, $SERVICE:expr,
        subscribe($FieldEnum:ty) -> $Event:ty,
        add($FieldEnum2:ty),
    ) => {
        impl $Struct {
            const SERVICE: &'static str = $SERVICE;

            pub(super) fn new(inner: Arc<StreamClientInner>) -> Self {
                Self { inner }
            }

            pub async fn subscribe(
                &self,
                symbols:  &[&str],
                fields:   &[$FieldEnum],
                capacity: usize,
            ) -> Result<mpsc::Receiver<$Event>> {
                let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
                let keys: Vec<String>   = symbols.iter().map(|s| s.to_string()).collect();

                let (raw_tx, mut raw_rx) = mpsc::channel::<serde_json::Value>(capacity);
                let (typed_tx, typed_rx) = mpsc::channel::<$Event>(capacity);

                let (reply_tx, reply_rx) = oneshot::channel();
                self.inner.cmd_tx.send(ActorCommand::Subscribe {
                    service: Self::SERVICE,
                    keys,
                    fields: field_ids,
                    raw_tx,
                    reply: reply_tx,
                }).await.map_err(|_| Error::StreamDisconnected)?;
                reply_rx.await.map_err(|_| Error::StreamDisconnected)??;

                tokio::spawn(async move {
                    while let Some(raw) = raw_rx.recv().await {
                        if let Ok(event) = <$Event>::try_from(&raw)
                            && typed_tx.send(event).await.is_err()
                        {
                            break;
                        }
                    }
                });

                Ok(typed_rx)
            }

            pub async fn add_symbols(
                &self,
                symbols: &[&str],
                fields:  &[$FieldEnum2],
            ) -> Result<()> {
                let field_ids: Vec<u32> = fields.iter().map(|f| *f as u32).collect();
                let keys: Vec<String>   = symbols.iter().map(|s| s.to_string()).collect();
                let (reply_tx, reply_rx) = oneshot::channel();
                self.inner.cmd_tx.send(ActorCommand::AddSymbols {
                    service: Self::SERVICE,
                    keys,
                    fields: field_ids,
                    reply: reply_tx,
                }).await.map_err(|_| Error::StreamDisconnected)?;
                reply_rx.await.map_err(|_| Error::StreamDisconnected)?
            }

            pub async fn unsubscribe(&self, symbols: &[&str]) -> Result<()> {
                let keys: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
                let (reply_tx, reply_rx) = oneshot::channel();
                self.inner.cmd_tx.send(ActorCommand::Unsubscribe {
                    service: Self::SERVICE,
                    keys,
                    reply: reply_tx,
                }).await.map_err(|_| Error::StreamDisconnected)?;
                reply_rx.await.map_err(|_| Error::StreamDisconnected)?
            }
        }
    };
}

// ── LevelOneEquitySub ─────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_EQUITIES` service.
pub struct LevelOneEquitySub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    LevelOneEquitySub, "LEVELONE_EQUITIES",
    subscribe(LevelOneEquityField) -> LevelOneEquityEvent,
    add(LevelOneEquityField),
);

// ── LevelOneOptionSub ─────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_OPTIONS` service.
pub struct LevelOneOptionSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    LevelOneOptionSub, "LEVELONE_OPTIONS",
    subscribe(LevelOneOptionField) -> LevelOneOptionEvent,
    add(LevelOneOptionField),
);

// ── LevelOneFuturesSub ────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FUTURES` service.
pub struct LevelOneFuturesSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    LevelOneFuturesSub, "LEVELONE_FUTURES",
    subscribe(LevelOneFuturesField) -> LevelOneFuturesEvent,
    add(LevelOneFuturesField),
);

// ── LevelOneForexSub ──────────────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FOREX` service.
pub struct LevelOneForexSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    LevelOneForexSub, "LEVELONE_FOREX",
    subscribe(LevelOneForexField) -> LevelOneForexEvent,
    add(LevelOneForexField),
);

// ── LevelOneFuturesOptionsSub ─────────────────────────────────────────────────

/// Subscription helper for the `LEVELONE_FUTURES_OPTIONS` service.
pub struct LevelOneFuturesOptionsSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    LevelOneFuturesOptionsSub, "LEVELONE_FUTURES_OPTIONS",
    subscribe(LevelOneFuturesOptionField) -> LevelOneFuturesOptionsEvent,
    add(LevelOneFuturesOptionField),
);

// ── ChartEquitySub ────────────────────────────────────────────────────────────

/// Subscription helper for the `CHART_EQUITY` service.
pub struct ChartEquitySub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    ChartEquitySub, "CHART_EQUITY",
    subscribe(ChartEquityField) -> ChartEquityEvent,
    add(ChartEquityField),
);

// ── ChartFuturesSub ───────────────────────────────────────────────────────────

/// Subscription helper for the `CHART_FUTURES` service.
pub struct ChartFuturesSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    ChartFuturesSub, "CHART_FUTURES",
    subscribe(ChartFuturesField) -> ChartFuturesEvent,
    add(ChartFuturesField),
);

// ── NyseBookSub ───────────────────────────────────────────────────────────────

/// Subscription helper for the `NYSE_BOOK` service.
pub struct NyseBookSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    NyseBookSub, "NYSE_BOOK",
    subscribe(BookField) -> BookEvent,
    add(BookField),
);

// ── NasdaqBookSub ─────────────────────────────────────────────────────────────

/// Subscription helper for the `NASDAQ_BOOK` service.
pub struct NasdaqBookSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    NasdaqBookSub, "NASDAQ_BOOK",
    subscribe(BookField) -> BookEvent,
    add(BookField),
);

// ── OptionsBookSub ────────────────────────────────────────────────────────────

/// Subscription helper for the `OPTIONS_BOOK` service.
pub struct OptionsBookSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    OptionsBookSub, "OPTIONS_BOOK",
    subscribe(BookField) -> BookEvent,
    add(BookField),
);

// ── ScreenerEquitySub ─────────────────────────────────────────────────────────

/// Subscription helper for the `SCREENER_EQUITY` service.
pub struct ScreenerEquitySub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    ScreenerEquitySub, "SCREENER_EQUITY",
    subscribe(ScreenerField) -> ScreenerEvent,
    add(ScreenerField),
);

// ── ScreenerOptionSub ─────────────────────────────────────────────────────────

/// Subscription helper for the `SCREENER_OPTION` service.
pub struct ScreenerOptionSub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    ScreenerOptionSub, "SCREENER_OPTION",
    subscribe(ScreenerField) -> ScreenerEvent,
    add(ScreenerField),
);

// ── AccountActivitySub ────────────────────────────────────────────────────────

/// Subscription helper for the `ACCT_ACTIVITY` service.
pub struct AccountActivitySub { inner: Arc<StreamClientInner> }

impl_subscribe!(
    AccountActivitySub, "ACCT_ACTIVITY",
    subscribe(AccountActivityField) -> AccountActivityEvent,
    add(AccountActivityField),
);
