//! Quote models for equity, option, forex, future, and index instruments.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Optional fields to include in a quote response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteFields {
    /// Include only the fundamental data.
    Fundamental,
    /// Include extended market data.
    Extended,
    /// Include reference data.
    Reference,
    /// Include regular market data.
    Regular,
    /// Include all available fields.
    All,
    /// Include only the quote snapshot.
    Quote,
}

impl QuoteFields {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            QuoteFields::Fundamental => "fundamental",
            QuoteFields::Extended => "extended",
            QuoteFields::Reference => "reference",
            QuoteFields::Regular => "regular",
            QuoteFields::All => "all",
            QuoteFields::Quote => "quote",
        }
    }
}

/// Top-level response map from `GET /marketdata/v1/quotes`.
pub type QuotesMap = HashMap<String, QuoteResponse>;

/// A single quote entry — discriminated by `assetMainType`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "assetMainType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuoteResponse {
    Equity(Box<EquityQuoteResponse>),
    Option(Box<OptionQuoteResponse>),
    Forex(Box<ForexQuoteResponse>),
    Future(Box<FutureQuoteResponse>),
    FutureOption(Box<FutureOptionQuoteResponse>),
    Index(Box<IndexQuoteResponse>),
    #[serde(other)]
    Unknown,
}

/// Equity quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityQuoteResponse {
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub realtime: Option<bool>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub quote: Option<EquityQuote>,
    pub reference: Option<EquityReference>,
    pub regular: Option<RegularMarket>,
    pub extended: Option<ExtendedMarket>,
    pub fundamental: Option<Fundamental>,
}

/// Intraday equity quote snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityQuote {
    pub ask_mic_id: Option<String>,
    pub ask_price: Option<f64>,
    pub ask_size: Option<i64>,
    pub ask_time: Option<i64>,
    pub bid_mic_id: Option<String>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i64>,
    pub bid_time: Option<i64>,
    pub close_price: Option<f64>,
    pub high_price: Option<f64>,
    pub last_mic_id: Option<String>,
    pub last_price: Option<f64>,
    pub last_size: Option<i64>,
    pub low_price: Option<f64>,
    pub mark: Option<f64>,
    pub mark_change: Option<f64>,
    pub mark_percent_change: Option<f64>,
    pub net_change: Option<f64>,
    pub net_percent_change: Option<f64>,
    pub open_price: Option<f64>,
    pub post_market_change: Option<f64>,
    pub post_market_percent_change: Option<f64>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub volatility: Option<f64>,
    pub week52_high: Option<f64>,
    pub week52_low: Option<f64>,
}

/// Reference / instrument metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityReference {
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub is_hard_to_borrow: Option<bool>,
    pub is_shortable: Option<bool>,
    pub htb_rate: Option<f64>,
}

/// Regular trading session summary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegularMarket {
    pub regular_market_last_price: Option<f64>,
    pub regular_market_last_size: Option<i64>,
    pub regular_market_net_change: Option<f64>,
    pub regular_market_percent_change: Option<f64>,
    pub regular_market_trade_time: Option<i64>,
}

/// Extended-hours trading summary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedMarket {
    pub ask_price: Option<f64>,
    pub ask_size: Option<i64>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i64>,
    pub last_price: Option<f64>,
    pub last_size: Option<i64>,
    pub mark: Option<f64>,
    pub quote_time: Option<i64>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Fundamental data for an instrument.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Fundamental {
    pub avg10_days_volume: Option<f64>,
    pub avg1_year_volume: Option<f64>,
    pub declaration_date: Option<String>,
    pub div_amount: Option<f64>,
    pub div_ex_date: Option<String>,
    pub div_freq: Option<i32>,
    pub div_pay_amount: Option<f64>,
    pub div_pay_date: Option<String>,
    pub div_yield: Option<f64>,
    pub eps: Option<f64>,
    pub fund_leverage_factor: Option<f64>,
    pub last_earnings_date: Option<String>,
    pub next_div_ex_date: Option<String>,
    pub next_div_pay_date: Option<String>,
    pub pe_ratio: Option<f64>,
}

/// Option contract quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionQuoteResponse {
    pub asset_main_type: Option<String>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub realtime: Option<bool>,
    pub quote: Option<OptionQuote>,
    pub reference: Option<OptionReference>,
}

/// Option contract intraday quote.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionQuote {
    pub ask_price: Option<f64>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i32>,
    pub close_price: Option<f64>,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub high_price: Option<f64>,
    pub ind_ask_price: Option<f64>,
    pub ind_bid_price: Option<f64>,
    pub ind_quote_time: Option<i64>,
    pub implied_yield: Option<f64>,
    pub last_price: Option<f64>,
    pub last_size: Option<i32>,
    pub low_price: Option<f64>,
    pub mark: Option<f64>,
    pub mark_change: Option<f64>,
    pub mark_percent_change: Option<f64>,
    pub money_intrinsic_value: Option<f64>,
    pub net_change: Option<f64>,
    pub net_percent_change: Option<f64>,
    pub open_interest: Option<f64>,
    pub open_price: Option<f64>,
    pub quote_time: Option<i64>,
    pub rho: Option<f64>,
    pub security_status: Option<String>,
    pub theoretical_option_value: Option<f64>,
    pub theta: Option<f64>,
    pub time_value: Option<f64>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub vega: Option<f64>,
    pub volatility: Option<f64>,
}

/// Option contract reference data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionReference {
    pub contract_type: Option<String>,
    pub cusip: Option<String>,
    pub days_to_expiration: Option<i32>,
    pub deliverables: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub exercise_type: Option<String>,
    pub expiration_day: Option<i32>,
    pub expiration_month: Option<i32>,
    pub expiration_type: Option<String>,
    pub expiration_year: Option<i32>,
    pub is_penny_pilot: Option<bool>,
    pub last_trading_day: Option<i64>,
    pub multiplier: Option<f64>,
    pub settlement_type: Option<String>,
    pub strike_price: Option<f64>,
    pub underlying: Option<String>,
    pub uv_expiration_type: Option<String>,
}

/// Forex quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForexQuoteResponse {
    pub asset_main_type: Option<String>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub realtime: Option<bool>,
    pub quote: Option<ForexQuote>,
    pub reference: Option<ForexReference>,
}

/// Forex intraday quote.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForexQuote {
    pub ask_price: Option<f64>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i32>,
    pub close_price: Option<f64>,
    pub high_price: Option<f64>,
    pub last_price: Option<f64>,
    pub last_size: Option<i32>,
    pub low_price: Option<f64>,
    pub mark: Option<f64>,
    pub net_change: Option<f64>,
    pub net_percent_change: Option<f64>,
    pub open_price: Option<f64>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub tick: Option<f64>,
    pub tick_amount: Option<f64>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Forex reference data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForexReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub is_tradable: Option<bool>,
    pub market_maker: Option<String>,
    pub product: Option<String>,
    pub trading_hours: Option<String>,
}

/// Futures quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureQuoteResponse {
    pub asset_main_type: Option<String>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub realtime: Option<bool>,
    pub quote: Option<FutureQuote>,
    pub reference: Option<FutureReference>,
}

/// Futures intraday quote.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureQuote {
    pub ask_mic_id: Option<String>,
    pub ask_price: Option<f64>,
    pub ask_size: Option<i32>,
    pub ask_time: Option<i64>,
    pub bid_mic_id: Option<String>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i32>,
    pub bid_time: Option<i64>,
    pub close_price: Option<f64>,
    pub future_percent_change: Option<f64>,
    pub high_price: Option<f64>,
    pub last_mic_id: Option<String>,
    pub last_price: Option<f64>,
    pub last_size: Option<i32>,
    pub low_price: Option<f64>,
    pub mark: Option<f64>,
    pub net_change: Option<f64>,
    pub open_interest: Option<f64>,
    pub open_price: Option<f64>,
    pub quote_time: Option<i64>,
    pub quoted_in_session: Option<bool>,
    pub security_status: Option<String>,
    pub settlement_price: Option<f64>,
    pub tick: Option<f64>,
    pub tick_amount: Option<f64>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Futures contract reference data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub future_is_active: Option<bool>,
    pub future_expiration_date: Option<i64>,
    pub future_multiplier: Option<f64>,
    pub future_price_format: Option<String>,
    pub future_settlement_price: Option<f64>,
    pub future_trading_hours: Option<String>,
    pub product: Option<String>,
}

/// Futures option quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureOptionQuoteResponse {
    pub asset_main_type: Option<String>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub realtime: Option<bool>,
    pub quote: Option<FutureOptionQuote>,
    pub reference: Option<FutureOptionReference>,
}

/// Futures option intraday quote.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureOptionQuote {
    pub ask_price: Option<f64>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i32>,
    pub close_price: Option<f64>,
    pub high_price: Option<f64>,
    pub last_price: Option<f64>,
    pub last_size: Option<i32>,
    pub low_price: Option<f64>,
    pub mark: Option<f64>,
    pub mark_change: Option<f64>,
    pub net_change: Option<f64>,
    pub net_percent_change: Option<f64>,
    pub open_interest: Option<f64>,
    pub open_price: Option<f64>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub settlement_price: Option<f64>,
    pub tick: Option<f64>,
    pub tick_amount: Option<f64>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub volatility: Option<f64>,
}

/// Futures option reference data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureOptionReference {
    pub contract_type: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub expiration_day: Option<i32>,
    pub expiration_month: Option<i32>,
    pub expiration_style: Option<String>,
    pub expiration_year: Option<i32>,
    pub is_penny_pilot: Option<bool>,
    pub last_trading_day: Option<i64>,
    pub multiplier: Option<f64>,
    pub settlement_type: Option<String>,
    pub strike_price: Option<f64>,
    pub underlying: Option<String>,
}

/// Index quote response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexQuoteResponse {
    pub asset_main_type: Option<String>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
    pub realtime: Option<bool>,
    pub quote: Option<IndexQuote>,
    pub reference: Option<IndexReference>,
}

/// Index intraday quote.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexQuote {
    pub close_price: Option<f64>,
    pub high_price: Option<f64>,
    pub last_price: Option<f64>,
    pub low_price: Option<f64>,
    pub net_change: Option<f64>,
    pub net_percent_change: Option<f64>,
    pub open_price: Option<f64>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub week52_high: Option<f64>,
    pub week52_low: Option<f64>,
}

/// Index reference data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
}
