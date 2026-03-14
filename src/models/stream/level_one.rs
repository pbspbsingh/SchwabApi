//! Level One streaming event structs.
//!
//! Each event is a partial update — only the changed fields are present.
//! All fields except `symbol` are `Option<T>`. Callers that need a full
//! snapshot must maintain their own state map and merge incoming updates.

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::error::{Error, Result};

// ── helpers ──────────────────────────────────────────────────────────────────

fn millis_to_dt(v: &Value) -> Option<DateTime<Utc>> {
    v.as_i64()
        .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
}

fn as_f64(v: &Value) -> Option<f64> {
    v.as_f64()
}

fn as_i64(v: &Value) -> Option<i64> {
    v.as_i64()
}

fn as_u64(v: &Value) -> Option<u64> {
    v.as_u64()
}

fn as_bool(v: &Value) -> Option<bool> {
    v.as_bool()
}

fn as_string(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.to_string())
}

// ── LevelOneEquityEvent ───────────────────────────────────────────────────────

/// A streaming update for a single equity.
///
/// Fields correspond to the numeric indices in the Schwab streaming spec.
/// Only changed fields are included in each message; all non-symbol fields
/// are `Option<T>`.
#[derive(Debug, Clone, Default)]
pub struct LevelOneEquityEvent {
    /// Field 0 — ticker symbol (always present).
    pub symbol: String,
    /// Field 1
    pub bid_price: Option<f64>,
    /// Field 2
    pub ask_price: Option<f64>,
    /// Field 3
    pub last_price: Option<f64>,
    /// Field 4
    pub bid_size: Option<u64>,
    /// Field 5
    pub ask_size: Option<u64>,
    /// Field 6
    pub ask_id: Option<String>,
    /// Field 7
    pub bid_id: Option<String>,
    /// Field 8
    pub total_volume: Option<u64>,
    /// Field 9
    pub last_size: Option<u64>,
    /// Field 10 — milliseconds since epoch
    pub quote_time: Option<DateTime<Utc>>,
    /// Field 11 — milliseconds since epoch
    pub trade_time: Option<DateTime<Utc>>,
    /// Field 12
    pub high_price: Option<f64>,
    /// Field 13
    pub low_price: Option<f64>,
    /// Field 14
    pub bid_tick: Option<String>,
    /// Field 15
    pub close_price: Option<f64>,
    /// Field 16
    pub exchange_id: Option<String>,
    /// Field 17
    pub marginable: Option<bool>,
    /// Field 18
    pub shortable: Option<bool>,
    /// Field 19
    pub island_bid_price: Option<f64>,
    /// Field 20
    pub island_ask_price: Option<f64>,
    /// Field 21
    pub island_volume: Option<u64>,
    /// Field 22
    pub quote_day: Option<i64>,
    /// Field 23
    pub trade_day: Option<i64>,
    /// Field 24
    pub volatility: Option<f64>,
    /// Field 25
    pub description: Option<String>,
    /// Field 26
    pub last_id: Option<String>,
    /// Field 27
    pub digits: Option<i64>,
    /// Field 28
    pub open_price: Option<f64>,
    /// Field 29
    pub net_change: Option<f64>,
    /// Field 30
    pub week52_high: Option<f64>,
    /// Field 31
    pub week52_low: Option<f64>,
    /// Field 32
    pub pe_ratio: Option<f64>,
    /// Field 33
    pub dividend_amount: Option<f64>,
    /// Field 34
    pub dividend_yield: Option<f64>,
    /// Field 35
    pub island_bid_size: Option<u64>,
    /// Field 36
    pub island_ask_size: Option<u64>,
    /// Field 37
    pub nav: Option<f64>,
    /// Field 38
    pub fund_price: Option<f64>,
    /// Field 39
    pub exchange_name: Option<String>,
    /// Field 40
    pub dividend_date: Option<String>,
    /// Field 41
    pub is_regular_market_quote: Option<bool>,
    /// Field 42
    pub is_regular_market_trade: Option<bool>,
    /// Field 43 — milliseconds since epoch
    pub regular_market_last_price: Option<f64>,
    /// Field 44
    pub regular_market_last_size: Option<u64>,
    /// Field 45 — milliseconds since epoch
    pub regular_market_trade_time: Option<DateTime<Utc>>,
    /// Field 46 — milliseconds since epoch
    pub regular_market_trade_day: Option<i64>,
    /// Field 47
    pub regular_market_net_change: Option<f64>,
    /// Field 48
    pub security_status: Option<String>,
    /// Field 49
    pub mark: Option<f64>,
    /// Field 50 — milliseconds since epoch
    pub quote_time_millis: Option<DateTime<Utc>>,
    /// Field 51 — milliseconds since epoch
    pub trade_time_millis: Option<DateTime<Utc>>,
}

impl TryFrom<&Value> for LevelOneEquityEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Json(serde_json::from_str::<()>("").unwrap_err()))?;
        let mut e = LevelOneEquityEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0"  => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1"  => e.bid_price = as_f64(val),
                "2"  => e.ask_price = as_f64(val),
                "3"  => e.last_price = as_f64(val),
                "4"  => e.bid_size = as_u64(val),
                "5"  => e.ask_size = as_u64(val),
                "6"  => e.ask_id = as_string(val),
                "7"  => e.bid_id = as_string(val),
                "8"  => e.total_volume = as_u64(val),
                "9"  => e.last_size = as_u64(val),
                "10" => e.quote_time = millis_to_dt(val),
                "11" => e.trade_time = millis_to_dt(val),
                "12" => e.high_price = as_f64(val),
                "13" => e.low_price = as_f64(val),
                "14" => e.bid_tick = as_string(val),
                "15" => e.close_price = as_f64(val),
                "16" => e.exchange_id = as_string(val),
                "17" => e.marginable = as_bool(val),
                "18" => e.shortable = as_bool(val),
                "19" => e.island_bid_price = as_f64(val),
                "20" => e.island_ask_price = as_f64(val),
                "21" => e.island_volume = as_u64(val),
                "22" => e.quote_day = as_i64(val),
                "23" => e.trade_day = as_i64(val),
                "24" => e.volatility = as_f64(val),
                "25" => e.description = as_string(val),
                "26" => e.last_id = as_string(val),
                "27" => e.digits = as_i64(val),
                "28" => e.open_price = as_f64(val),
                "29" => e.net_change = as_f64(val),
                "30" => e.week52_high = as_f64(val),
                "31" => e.week52_low = as_f64(val),
                "32" => e.pe_ratio = as_f64(val),
                "33" => e.dividend_amount = as_f64(val),
                "34" => e.dividend_yield = as_f64(val),
                "35" => e.island_bid_size = as_u64(val),
                "36" => e.island_ask_size = as_u64(val),
                "37" => e.nav = as_f64(val),
                "38" => e.fund_price = as_f64(val),
                "39" => e.exchange_name = as_string(val),
                "40" => e.dividend_date = as_string(val),
                "41" => e.is_regular_market_quote = as_bool(val),
                "42" => e.is_regular_market_trade = as_bool(val),
                "43" => e.regular_market_last_price = as_f64(val),
                "44" => e.regular_market_last_size = as_u64(val),
                "45" => e.regular_market_trade_time = millis_to_dt(val),
                "46" => e.regular_market_trade_day = as_i64(val),
                "47" => e.regular_market_net_change = as_f64(val),
                "48" => e.security_status = as_string(val),
                "49" => e.mark = as_f64(val),
                "50" => e.quote_time_millis = millis_to_dt(val),
                "51" => e.trade_time_millis = millis_to_dt(val),
                _    => {}
            }
        }
        Ok(e)
    }
}

// ── LevelOneOptionEvent ───────────────────────────────────────────────────────

/// A streaming update for a single option contract.
#[derive(Debug, Clone, Default)]
pub struct LevelOneOptionEvent {
    /// Field 0 — option symbol (always present).
    pub symbol: String,
    /// Field 1
    pub description: Option<String>,
    /// Field 2
    pub bid_price: Option<f64>,
    /// Field 3
    pub ask_price: Option<f64>,
    /// Field 4
    pub last_price: Option<f64>,
    /// Field 5
    pub high_price: Option<f64>,
    /// Field 6
    pub low_price: Option<f64>,
    /// Field 7
    pub close_price: Option<f64>,
    /// Field 8
    pub total_volume: Option<u64>,
    /// Field 9
    pub open_interest: Option<i64>,
    /// Field 10
    pub volatility: Option<f64>,
    /// Field 11 — milliseconds
    pub quote_time: Option<DateTime<Utc>>,
    /// Field 12 — milliseconds
    pub trade_time: Option<DateTime<Utc>>,
    /// Field 13
    pub money_intrinsic_value: Option<f64>,
    /// Field 14
    pub quote_day: Option<i64>,
    /// Field 15
    pub trade_day: Option<i64>,
    /// Field 16
    pub expiration_year: Option<i64>,
    /// Field 17
    pub multiplier: Option<f64>,
    /// Field 18
    pub digits: Option<i64>,
    /// Field 19
    pub open_price: Option<f64>,
    /// Field 20
    pub bid_size: Option<u64>,
    /// Field 21
    pub ask_size: Option<u64>,
    /// Field 22
    pub last_size: Option<u64>,
    /// Field 23
    pub net_change: Option<f64>,
    /// Field 24
    pub strike_price: Option<f64>,
    /// Field 25
    pub contract_type: Option<String>,
    /// Field 26
    pub underlying: Option<String>,
    /// Field 27
    pub expiration_month: Option<i64>,
    /// Field 28
    pub deliverables: Option<String>,
    /// Field 29
    pub time_value: Option<f64>,
    /// Field 30
    pub expiration_day: Option<i64>,
    /// Field 31
    pub days_to_expiration: Option<i64>,
    /// Field 32
    pub delta: Option<f64>,
    /// Field 33
    pub gamma: Option<f64>,
    /// Field 34
    pub theta: Option<f64>,
    /// Field 35
    pub vega: Option<f64>,
    /// Field 36
    pub rho: Option<f64>,
    /// Field 37
    pub security_status: Option<String>,
    /// Field 38
    pub theoretical_option_value: Option<f64>,
    /// Field 39
    pub underlying_price: Option<f64>,
    /// Field 40
    pub uv_expiration_type: Option<String>,
    /// Field 41
    pub mark: Option<f64>,
    /// Field 42 — milliseconds
    pub quote_time_millis: Option<DateTime<Utc>>,
    /// Field 43 — milliseconds
    pub trade_time_millis: Option<DateTime<Utc>>,
    /// Field 44
    pub exchange_id: Option<String>,
    /// Field 45
    pub exchange_name: Option<String>,
    /// Field 46 — milliseconds
    pub last_trading_day: Option<i64>,
    /// Field 47
    pub settlement_type: Option<String>,
    /// Field 48
    pub net_percent_change: Option<f64>,
    /// Field 49
    pub mark_change: Option<f64>,
    /// Field 50
    pub mark_percent_change: Option<f64>,
    /// Field 51
    pub implied_yield: Option<f64>,
    /// Field 52
    pub is_penny_pilot: Option<bool>,
    /// Field 53
    pub option_root: Option<String>,
    /// Field 54
    pub week52_high: Option<f64>,
    /// Field 55
    pub week52_low: Option<f64>,
}

impl TryFrom<&Value> for LevelOneOptionEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Json(serde_json::from_str::<()>("").unwrap_err()))?;
        let mut e = LevelOneOptionEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0"  => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1"  => e.description = as_string(val),
                "2"  => e.bid_price = as_f64(val),
                "3"  => e.ask_price = as_f64(val),
                "4"  => e.last_price = as_f64(val),
                "5"  => e.high_price = as_f64(val),
                "6"  => e.low_price = as_f64(val),
                "7"  => e.close_price = as_f64(val),
                "8"  => e.total_volume = as_u64(val),
                "9"  => e.open_interest = as_i64(val),
                "10" => e.volatility = as_f64(val),
                "11" => e.quote_time = millis_to_dt(val),
                "12" => e.trade_time = millis_to_dt(val),
                "13" => e.money_intrinsic_value = as_f64(val),
                "14" => e.quote_day = as_i64(val),
                "15" => e.trade_day = as_i64(val),
                "16" => e.expiration_year = as_i64(val),
                "17" => e.multiplier = as_f64(val),
                "18" => e.digits = as_i64(val),
                "19" => e.open_price = as_f64(val),
                "20" => e.bid_size = as_u64(val),
                "21" => e.ask_size = as_u64(val),
                "22" => e.last_size = as_u64(val),
                "23" => e.net_change = as_f64(val),
                "24" => e.strike_price = as_f64(val),
                "25" => e.contract_type = as_string(val),
                "26" => e.underlying = as_string(val),
                "27" => e.expiration_month = as_i64(val),
                "28" => e.deliverables = as_string(val),
                "29" => e.time_value = as_f64(val),
                "30" => e.expiration_day = as_i64(val),
                "31" => e.days_to_expiration = as_i64(val),
                "32" => e.delta = as_f64(val),
                "33" => e.gamma = as_f64(val),
                "34" => e.theta = as_f64(val),
                "35" => e.vega = as_f64(val),
                "36" => e.rho = as_f64(val),
                "37" => e.security_status = as_string(val),
                "38" => e.theoretical_option_value = as_f64(val),
                "39" => e.underlying_price = as_f64(val),
                "40" => e.uv_expiration_type = as_string(val),
                "41" => e.mark = as_f64(val),
                "42" => e.quote_time_millis = millis_to_dt(val),
                "43" => e.trade_time_millis = millis_to_dt(val),
                "44" => e.exchange_id = as_string(val),
                "45" => e.exchange_name = as_string(val),
                "46" => e.last_trading_day = as_i64(val),
                "47" => e.settlement_type = as_string(val),
                "48" => e.net_percent_change = as_f64(val),
                "49" => e.mark_change = as_f64(val),
                "50" => e.mark_percent_change = as_f64(val),
                "51" => e.implied_yield = as_f64(val),
                "52" => e.is_penny_pilot = as_bool(val),
                "53" => e.option_root = as_string(val),
                "54" => e.week52_high = as_f64(val),
                "55" => e.week52_low = as_f64(val),
                _    => {}
            }
        }
        Ok(e)
    }
}

// ── LevelOneFuturesEvent ──────────────────────────────────────────────────────

/// A streaming update for a single futures contract.
#[derive(Debug, Clone, Default)]
pub struct LevelOneFuturesEvent {
    /// Field 0 — futures symbol (always present).
    pub symbol: String,
    /// Field 1
    pub bid_price: Option<f64>,
    /// Field 2
    pub ask_price: Option<f64>,
    /// Field 3
    pub last_price: Option<f64>,
    /// Field 4
    pub bid_size: Option<i64>,
    /// Field 5
    pub ask_size: Option<i64>,
    /// Field 6
    pub ask_id: Option<String>,
    /// Field 7
    pub bid_id: Option<String>,
    /// Field 8
    pub total_volume: Option<u64>,
    /// Field 9
    pub last_size: Option<i64>,
    /// Field 10 — milliseconds
    pub quote_time: Option<DateTime<Utc>>,
    /// Field 11 — milliseconds
    pub trade_time: Option<DateTime<Utc>>,
    /// Field 12
    pub high_price: Option<f64>,
    /// Field 13
    pub low_price: Option<f64>,
    /// Field 14
    pub close_price: Option<f64>,
    /// Field 15
    pub exchange_id: Option<String>,
    /// Field 16
    pub description: Option<String>,
    /// Field 17
    pub last_id: Option<String>,
    /// Field 18
    pub open_price: Option<f64>,
    /// Field 19
    pub net_change: Option<f64>,
    /// Field 20
    pub future_percent_change: Option<f64>,
    /// Field 21
    pub exchange_name: Option<String>,
    /// Field 22
    pub security_status: Option<String>,
    /// Field 23
    pub open_interest: Option<i64>,
    /// Field 24
    pub mark: Option<f64>,
    /// Field 25
    pub tick: Option<f64>,
    /// Field 26
    pub tick_amount: Option<f64>,
    /// Field 27
    pub product: Option<String>,
    /// Field 28
    pub future_price_format: Option<String>,
    /// Field 29
    pub future_trading_hours: Option<String>,
    /// Field 30
    pub future_is_tradable: Option<bool>,
    /// Field 31
    pub future_multiplier: Option<f64>,
    /// Field 32
    pub future_is_active: Option<bool>,
    /// Field 33
    pub future_settlement_price: Option<f64>,
    /// Field 34
    pub future_active_symbol: Option<String>,
    /// Field 35 — milliseconds
    pub future_expiration_date: Option<DateTime<Utc>>,
    /// Field 36
    pub expiration_style: Option<String>,
    /// Field 37
    pub ask_time: Option<DateTime<Utc>>,
    /// Field 38
    pub bid_time: Option<DateTime<Utc>>,
    /// Field 39
    pub quoted_in_session: Option<bool>,
    /// Field 40
    pub settlement_date: Option<DateTime<Utc>>,
}

impl TryFrom<&Value> for LevelOneFuturesEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Json(serde_json::from_str::<()>("").unwrap_err()))?;
        let mut e = LevelOneFuturesEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0"  => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1"  => e.bid_price = as_f64(val),
                "2"  => e.ask_price = as_f64(val),
                "3"  => e.last_price = as_f64(val),
                "4"  => e.bid_size = as_i64(val),
                "5"  => e.ask_size = as_i64(val),
                "6"  => e.ask_id = as_string(val),
                "7"  => e.bid_id = as_string(val),
                "8"  => e.total_volume = as_u64(val),
                "9"  => e.last_size = as_i64(val),
                "10" => e.quote_time = millis_to_dt(val),
                "11" => e.trade_time = millis_to_dt(val),
                "12" => e.high_price = as_f64(val),
                "13" => e.low_price = as_f64(val),
                "14" => e.close_price = as_f64(val),
                "15" => e.exchange_id = as_string(val),
                "16" => e.description = as_string(val),
                "17" => e.last_id = as_string(val),
                "18" => e.open_price = as_f64(val),
                "19" => e.net_change = as_f64(val),
                "20" => e.future_percent_change = as_f64(val),
                "21" => e.exchange_name = as_string(val),
                "22" => e.security_status = as_string(val),
                "23" => e.open_interest = as_i64(val),
                "24" => e.mark = as_f64(val),
                "25" => e.tick = as_f64(val),
                "26" => e.tick_amount = as_f64(val),
                "27" => e.product = as_string(val),
                "28" => e.future_price_format = as_string(val),
                "29" => e.future_trading_hours = as_string(val),
                "30" => e.future_is_tradable = as_bool(val),
                "31" => e.future_multiplier = as_f64(val),
                "32" => e.future_is_active = as_bool(val),
                "33" => e.future_settlement_price = as_f64(val),
                "34" => e.future_active_symbol = as_string(val),
                "35" => e.future_expiration_date = millis_to_dt(val),
                "36" => e.expiration_style = as_string(val),
                "37" => e.ask_time = millis_to_dt(val),
                "38" => e.bid_time = millis_to_dt(val),
                "39" => e.quoted_in_session = as_bool(val),
                "40" => e.settlement_date = millis_to_dt(val),
                _    => {}
            }
        }
        Ok(e)
    }
}

// ── LevelOneForexEvent ────────────────────────────────────────────────────────

/// A streaming update for a single forex pair.
#[derive(Debug, Clone, Default)]
pub struct LevelOneForexEvent {
    /// Field 0 — forex pair symbol (always present).
    pub symbol: String,
    /// Field 1
    pub bid_price: Option<f64>,
    /// Field 2
    pub ask_price: Option<f64>,
    /// Field 3
    pub last_price: Option<f64>,
    /// Field 4
    pub bid_size: Option<i64>,
    /// Field 5
    pub ask_size: Option<i64>,
    /// Field 6
    pub total_volume: Option<u64>,
    /// Field 7
    pub last_size: Option<i64>,
    /// Field 8 — milliseconds
    pub quote_time: Option<DateTime<Utc>>,
    /// Field 9 — milliseconds
    pub trade_time: Option<DateTime<Utc>>,
    /// Field 10
    pub high_price: Option<f64>,
    /// Field 11
    pub low_price: Option<f64>,
    /// Field 12
    pub close_price: Option<f64>,
    /// Field 13
    pub exchange_id: Option<String>,
    /// Field 14
    pub description: Option<String>,
    /// Field 15
    pub open_price: Option<f64>,
    /// Field 16
    pub net_change: Option<f64>,
    /// Field 17
    pub percent_change: Option<f64>,
    /// Field 18
    pub exchange_name: Option<String>,
    /// Field 19
    pub digits: Option<i64>,
    /// Field 20
    pub security_status: Option<String>,
    /// Field 21
    pub tick: Option<f64>,
    /// Field 22
    pub tick_amount: Option<f64>,
    /// Field 23
    pub product: Option<String>,
    /// Field 24
    pub trading_hours: Option<String>,
    /// Field 25
    pub is_tradable: Option<bool>,
    /// Field 26
    pub market_maker: Option<String>,
    /// Field 27
    pub week52_high: Option<f64>,
    /// Field 28
    pub week52_low: Option<f64>,
    /// Field 29
    pub mark: Option<f64>,
}

impl TryFrom<&Value> for LevelOneForexEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Json(serde_json::from_str::<()>("").unwrap_err()))?;
        let mut e = LevelOneForexEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0"  => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1"  => e.bid_price = as_f64(val),
                "2"  => e.ask_price = as_f64(val),
                "3"  => e.last_price = as_f64(val),
                "4"  => e.bid_size = as_i64(val),
                "5"  => e.ask_size = as_i64(val),
                "6"  => e.total_volume = as_u64(val),
                "7"  => e.last_size = as_i64(val),
                "8"  => e.quote_time = millis_to_dt(val),
                "9"  => e.trade_time = millis_to_dt(val),
                "10" => e.high_price = as_f64(val),
                "11" => e.low_price = as_f64(val),
                "12" => e.close_price = as_f64(val),
                "13" => e.exchange_id = as_string(val),
                "14" => e.description = as_string(val),
                "15" => e.open_price = as_f64(val),
                "16" => e.net_change = as_f64(val),
                "17" => e.percent_change = as_f64(val),
                "18" => e.exchange_name = as_string(val),
                "19" => e.digits = as_i64(val),
                "20" => e.security_status = as_string(val),
                "21" => e.tick = as_f64(val),
                "22" => e.tick_amount = as_f64(val),
                "23" => e.product = as_string(val),
                "24" => e.trading_hours = as_string(val),
                "25" => e.is_tradable = as_bool(val),
                "26" => e.market_maker = as_string(val),
                "27" => e.week52_high = as_f64(val),
                "28" => e.week52_low = as_f64(val),
                "29" => e.mark = as_f64(val),
                _    => {}
            }
        }
        Ok(e)
    }
}

// ── LevelOneFuturesOptionsEvent ───────────────────────────────────────────────

/// A streaming update for a single futures option contract.
#[derive(Debug, Clone, Default)]
pub struct LevelOneFuturesOptionsEvent {
    /// Field 0 — symbol (always present).
    pub symbol: String,
    /// Field 1
    pub bid_price: Option<f64>,
    /// Field 2
    pub ask_price: Option<f64>,
    /// Field 3
    pub last_price: Option<f64>,
    /// Field 4
    pub bid_size: Option<i64>,
    /// Field 5
    pub ask_size: Option<i64>,
    /// Field 6
    pub ask_id: Option<String>,
    /// Field 7
    pub bid_id: Option<String>,
    /// Field 8
    pub total_volume: Option<u64>,
    /// Field 9
    pub last_size: Option<i64>,
    /// Field 10 — milliseconds
    pub quote_time: Option<DateTime<Utc>>,
    /// Field 11 — milliseconds
    pub trade_time: Option<DateTime<Utc>>,
    /// Field 12
    pub high_price: Option<f64>,
    /// Field 13
    pub low_price: Option<f64>,
    /// Field 14
    pub close_price: Option<f64>,
    /// Field 15
    pub exchange_id: Option<String>,
    /// Field 16
    pub description: Option<String>,
    /// Field 17
    pub last_id: Option<String>,
    /// Field 18
    pub open_price: Option<f64>,
    /// Field 19
    pub net_change: Option<f64>,
    /// Field 20
    pub future_percent_change: Option<f64>,
    /// Field 21
    pub exchange_name: Option<String>,
    /// Field 22
    pub security_status: Option<String>,
    /// Field 23
    pub open_interest: Option<i64>,
    /// Field 24
    pub mark: Option<f64>,
    /// Field 25
    pub tick: Option<f64>,
    /// Field 26
    pub tick_amount: Option<f64>,
    /// Field 27
    pub product: Option<String>,
    /// Field 28
    pub future_price_format: Option<String>,
    /// Field 29
    pub future_trading_hours: Option<String>,
    /// Field 30
    pub future_is_tradable: Option<bool>,
    /// Field 31
    pub future_multiplier: Option<f64>,
}

impl TryFrom<&Value> for LevelOneFuturesOptionsEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Json(serde_json::from_str::<()>("").unwrap_err()))?;
        let mut e = LevelOneFuturesOptionsEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0"  => e.symbol = val.as_str().unwrap_or_default().to_string(),
                "1"  => e.bid_price = as_f64(val),
                "2"  => e.ask_price = as_f64(val),
                "3"  => e.last_price = as_f64(val),
                "4"  => e.bid_size = as_i64(val),
                "5"  => e.ask_size = as_i64(val),
                "6"  => e.ask_id = as_string(val),
                "7"  => e.bid_id = as_string(val),
                "8"  => e.total_volume = as_u64(val),
                "9"  => e.last_size = as_i64(val),
                "10" => e.quote_time = millis_to_dt(val),
                "11" => e.trade_time = millis_to_dt(val),
                "12" => e.high_price = as_f64(val),
                "13" => e.low_price = as_f64(val),
                "14" => e.close_price = as_f64(val),
                "15" => e.exchange_id = as_string(val),
                "16" => e.description = as_string(val),
                "17" => e.last_id = as_string(val),
                "18" => e.open_price = as_f64(val),
                "19" => e.net_change = as_f64(val),
                "20" => e.future_percent_change = as_f64(val),
                "21" => e.exchange_name = as_string(val),
                "22" => e.security_status = as_string(val),
                "23" => e.open_interest = as_i64(val),
                "24" => e.mark = as_f64(val),
                "25" => e.tick = as_f64(val),
                "26" => e.tick_amount = as_f64(val),
                "27" => e.product = as_string(val),
                "28" => e.future_price_format = as_string(val),
                "29" => e.future_trading_hours = as_string(val),
                "30" => e.future_is_tradable = as_bool(val),
                "31" => e.future_multiplier = as_f64(val),
                _    => {}
            }
        }
        Ok(e)
    }
}
