//! Chart (OHLCV bar) streaming event structs.

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::error::{Error, Result};

fn millis_to_dt(v: &Value) -> Option<DateTime<Utc>> {
    v.as_i64()
        .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
}

// ── ChartEquityEvent ──────────────────────────────────────────────────────────

/// A one-minute equity OHLCV bar from the CHART_EQUITY service.
#[derive(Debug, Clone, Default)]
pub struct ChartEquityEvent {
    /// Field 0 — symbol (always present).
    pub symbol: String,
    /// Field 1
    pub sequence: Option<i64>,
    /// Field 2
    pub open_price: Option<f64>,
    /// Field 3
    pub high_price: Option<f64>,
    /// Field 4
    pub low_price: Option<f64>,
    /// Field 5
    pub close_price: Option<f64>,
    /// Field 6
    pub volume: Option<f64>,
    /// Field 7 — milliseconds since epoch
    pub chart_time: Option<DateTime<Utc>>,
    /// Field 8
    pub chart_day: Option<i64>,
}

impl TryFrom<&Value> for ChartEquityEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Api { status: 0, body: "expected a JSON object".to_string() })?;
        let mut e = ChartEquityEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0" => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1" => e.sequence = val.as_i64(),
                "2" => e.open_price = val.as_f64(),
                "3" => e.high_price = val.as_f64(),
                "4" => e.low_price = val.as_f64(),
                "5" => e.close_price = val.as_f64(),
                "6" => e.volume = val.as_f64(),
                "7" => e.chart_time = millis_to_dt(val),
                "8" => e.chart_day = val.as_i64(),
                _   => {}
            }
        }
        Ok(e)
    }
}

// ── ChartFuturesEvent ─────────────────────────────────────────────────────────

/// A one-minute futures OHLCV bar from the CHART_FUTURES service.
#[derive(Debug, Clone, Default)]
pub struct ChartFuturesEvent {
    /// Field 0 — symbol (always present).
    pub symbol: String,
    /// Field 1 — milliseconds
    pub chart_time: Option<DateTime<Utc>>,
    /// Field 2
    pub open_price: Option<f64>,
    /// Field 3
    pub high_price: Option<f64>,
    /// Field 4
    pub low_price: Option<f64>,
    /// Field 5
    pub close_price: Option<f64>,
    /// Field 6
    pub volume: Option<f64>,
}

impl TryFrom<&Value> for ChartFuturesEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Api { status: 0, body: "expected a JSON object".to_string() })?;
        let mut e = ChartFuturesEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0" => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1" => e.chart_time = millis_to_dt(val),
                "2" => e.open_price = val.as_f64(),
                "3" => e.high_price = val.as_f64(),
                "4" => e.low_price = val.as_f64(),
                "5" => e.close_price = val.as_f64(),
                "6" => e.volume = val.as_f64(),
                _   => {}
            }
        }
        Ok(e)
    }
}
