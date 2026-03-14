//! Screener streaming event structs.

use serde_json::Value;

use crate::error::{Error, Result};

/// A single screener result entry.
#[derive(Debug, Clone, Default)]
pub struct ScreenerItem {
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub volume: Option<i64>,
    pub last_price: Option<f64>,
    pub net_change: Option<f64>,
    pub market_share: Option<f64>,
    pub trades: Option<i64>,
    pub net_percent_change: Option<f64>,
}

/// A streaming screener event from SCREENER_EQUITY or SCREENER_OPTION.
#[derive(Debug, Clone, Default)]
pub struct ScreenerEvent {
    /// Field 0 — screener key (always present).
    pub symbol: String,
    /// Field 1 — milliseconds
    pub timestamp: Option<i64>,
    /// Field 2
    pub sort_field: Option<String>,
    /// Field 3
    pub frequency: Option<i64>,
    /// Field 4 — the list of top screener entries.
    pub items: Vec<ScreenerItem>,
}

fn parse_screener_items(arr: &Value) -> Vec<ScreenerItem> {
    let mut items = Vec::new();
    if let Some(a) = arr.as_array() {
        for v in a {
            let mut item = ScreenerItem::default();
            if let Some(obj) = v.as_object() {
                for (k, val) in obj {
                    match k.as_str() {
                        "0" => item.symbol = val.as_str().map(|s| s.to_string()),
                        "1" => item.description = val.as_str().map(|s| s.to_string()),
                        "2" => item.volume = val.as_i64(),
                        "3" => item.last_price = val.as_f64(),
                        "4" => item.net_change = val.as_f64(),
                        "5" => item.market_share = val.as_f64(),
                        "6" => item.trades = val.as_i64(),
                        "7" => item.net_percent_change = val.as_f64(),
                        _   => {}
                    }
                }
            }
            items.push(item);
        }
    }
    items
}

impl TryFrom<&Value> for ScreenerEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Api { status: 0, body: "expected a JSON object".to_string() })?;
        let mut e = ScreenerEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0" => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1" => e.timestamp = val.as_i64(),
                "2" => e.sort_field = val.as_str().map(|s| s.to_string()),
                "3" => e.frequency = val.as_i64(),
                "4" => e.items = parse_screener_items(val),
                _   => {}
            }
        }
        Ok(e)
    }
}
