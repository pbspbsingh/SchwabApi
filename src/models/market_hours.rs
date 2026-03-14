//! Market hours models.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A market segment (exchange group) to query hours for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    Equity,
    Option,
    Future,
    Bond,
    Forex,
}

impl Market {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Market::Equity => "equity",
            Market::Option => "option",
            Market::Future => "future",
            Market::Bond => "bond",
            Market::Forex => "forex",
        }
    }
}

/// A single trading session window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHours {
    pub start: Option<String>,
    pub end: Option<String>,
}

/// Trading hours for one market on one date.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSession {
    pub date: Option<String>,
    pub market_type: Option<String>,
    pub exchange: Option<String>,
    pub category: Option<String>,
    pub product: Option<String>,
    pub product_name: Option<String>,
    pub is_open: Option<bool>,
    pub session_hours: Option<HashMap<String, Vec<SessionHours>>>,
}

/// Response from `GET /marketdata/v1/markets`.
///
/// Keyed outer map: market name → inner map: product symbol → hours.
pub type MarketHours = HashMap<String, HashMap<String, MarketSession>>;
