//! Level 2 order book streaming event structs.

use serde_json::Value;

use crate::error::{Error, Result};

/// A single price level in an order book.
#[derive(Debug, Clone, Default)]
pub struct BookLevel {
    pub price: Option<f64>,
    pub total_size: Option<i64>,
    pub total_count: Option<i64>,
    pub entries: Vec<BookEntry>,
}

/// A single exchange entry at a price level.
#[derive(Debug, Clone, Default)]
pub struct BookEntry {
    pub exchange: Option<String>,
    pub size: Option<i64>,
    pub sequence: Option<i64>,
}

fn parse_entries(arr: &Value) -> Vec<BookEntry> {
    let mut entries = Vec::new();
    if let Some(a) = arr.as_array() {
        for item in a {
            let mut entry = BookEntry::default();
            if let Some(obj) = item.as_object() {
                for (k, v) in obj {
                    match k.as_str() {
                        "0" => entry.exchange = v.as_str().map(|s| s.to_string()),
                        "1" => entry.size = v.as_i64(),
                        "2" => entry.sequence = v.as_i64(),
                        _   => {}
                    }
                }
            }
            entries.push(entry);
        }
    }
    entries
}

fn parse_levels(arr: &Value) -> Vec<BookLevel> {
    let mut levels = Vec::new();
    if let Some(a) = arr.as_array() {
        for item in a {
            let mut level = BookLevel::default();
            if let Some(obj) = item.as_object() {
                for (k, v) in obj {
                    match k.as_str() {
                        "0" => level.price = v.as_f64(),
                        "1" => level.total_size = v.as_i64(),
                        "2" => level.total_count = v.as_i64(),
                        "3" => level.entries = parse_entries(v),
                        _   => {}
                    }
                }
            }
            levels.push(level);
        }
    }
    levels
}

/// A Level 2 order book snapshot / delta event.
///
/// Used for NYSE_BOOK, NASDAQ_BOOK, and OPTIONS_BOOK services.
#[derive(Debug, Clone, Default)]
pub struct BookEvent {
    /// Field 0 — symbol (always present).
    pub symbol: String,
    /// Field 1 — milliseconds
    pub book_time: Option<i64>,
    /// Field 2 — bid side levels.
    pub bids: Vec<BookLevel>,
    /// Field 3 — ask side levels.
    pub asks: Vec<BookLevel>,
}

impl TryFrom<&Value> for BookEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Api { status: 0, body: "expected a JSON object".to_string() })?;
        let mut e = BookEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0" => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1" => e.book_time = val.as_i64(),
                "2" => e.bids = parse_levels(val),
                "3" => e.asks = parse_levels(val),
                _   => {}
            }
        }
        Ok(e)
    }
}
