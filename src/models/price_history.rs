//! Price history models and request builders.

use serde::{Deserialize, Serialize};

/// Granularity of each candlestick bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrequencyType {
    Minute,
    Daily,
    Weekly,
    Monthly,
}

impl FrequencyType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            FrequencyType::Minute => "minute",
            FrequencyType::Daily => "daily",
            FrequencyType::Weekly => "weekly",
            FrequencyType::Monthly => "monthly",
        }
    }
}

/// Period units for a price history request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PeriodType {
    Day,
    Month,
    Year,
    Ytd,
}

impl PeriodType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PeriodType::Day => "day",
            PeriodType::Month => "month",
            PeriodType::Year => "year",
            PeriodType::Ytd => "ytd",
        }
    }
}

/// All parameters accepted by `GET /marketdata/v1/pricehistory`.
#[derive(Debug, Clone, Default)]
pub struct PriceHistoryRequest {
    /// Ticker symbol (required).
    pub symbol: String,
    pub period_type: Option<PeriodType>,
    pub period: Option<i32>,
    pub frequency_type: Option<FrequencyType>,
    pub frequency: Option<i32>,
    /// Unix milliseconds — start of the desired range.
    pub start_date: Option<i64>,
    /// Unix milliseconds — end of the desired range.
    pub end_date: Option<i64>,
    /// Include extended-hours bars.
    pub need_extended_hours_data: Option<bool>,
    /// Include previous day's close for percent-change calculation.
    pub need_previous_close: Option<bool>,
}

/// A single OHLCV bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    /// Unix millisecond timestamp of the bar's open.
    pub datetime: i64,
    /// ISO-8601 string representation (optional, included by API).
    #[serde(rename = "datetimeISO8601")]
    pub datetime_iso8601: Option<String>,
}

/// Response from the price history endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub symbol: Option<String>,
    pub empty: Option<bool>,
    pub candles: Vec<Candle>,
    /// Previous close price (present when `needPreviousClose=true`).
    pub previous_close: Option<f64>,
    /// Timestamp of the previous close.
    pub previous_close_date: Option<i64>,
    pub previous_close_date_iso8601: Option<String>,
}
