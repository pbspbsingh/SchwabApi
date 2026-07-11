//! Option chain models and request builders.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::types::{Money, Timestamp};

/// Whether to include calls, puts, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Call,
    Put,
    All,
}

impl ContractType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContractType::Call => "CALL",
            ContractType::Put => "PUT",
            ContractType::All => "ALL",
        }
    }
}

/// Moneyness filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionRange {
    /// In-the-money only.
    Itm,
    /// Near-the-money only.
    Ntm,
    /// Out-of-the-money only.
    Otm,
    /// Standard (non-mini) contracts.
    Sak,
    /// Standard, mini, and non-standard.
    Sbk,
    /// Standard and non-standard.
    Snk,
    /// All contracts.
    All,
}

impl OptionRange {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            OptionRange::Itm => "ITM",
            OptionRange::Ntm => "NTM",
            OptionRange::Otm => "OTM",
            OptionRange::Sak => "SAK",
            OptionRange::Sbk => "SBK",
            OptionRange::Snk => "SNK",
            OptionRange::All => "ALL",
        }
    }
}

/// How expiration dates are grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpirationMonth {
    Jan, Feb, Mar, Apr, May, Jun,
    Jul, Aug, Sep, Oct, Nov, Dec,
    All,
}

impl ExpirationMonth {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ExpirationMonth::Jan => "JAN",
            ExpirationMonth::Feb => "FEB",
            ExpirationMonth::Mar => "MAR",
            ExpirationMonth::Apr => "APR",
            ExpirationMonth::May => "MAY",
            ExpirationMonth::Jun => "JUN",
            ExpirationMonth::Jul => "JUL",
            ExpirationMonth::Aug => "AUG",
            ExpirationMonth::Sep => "SEP",
            ExpirationMonth::Oct => "OCT",
            ExpirationMonth::Nov => "NOV",
            ExpirationMonth::Dec => "DEC",
            ExpirationMonth::All => "ALL",
        }
    }
}

/// Option type (standard vs. non-standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionType {
    /// Standard (exchange-listed) contracts only.
    Standard,
    /// Non-standard (flex, adjusted, etc.) only.
    NonStandard,
    /// All option types.
    All,
}

impl OptionType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            OptionType::Standard => "S",
            OptionType::NonStandard => "NS",
            OptionType::All => "ALL",
        }
    }
}

/// Strategy for which to return the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionStrategy {
    Single,
    AnalyticalJumbo,
    Covered,
    Vertical,
    Calendar,
    Strangle,
    Straddle,
    Butterfly,
    Condor,
    Diagonal,
    Collar,
    Roll,
}

impl OptionStrategy {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            OptionStrategy::Single => "SINGLE",
            OptionStrategy::AnalyticalJumbo => "ANALYTICAL",
            OptionStrategy::Covered => "COVERED",
            OptionStrategy::Vertical => "VERTICAL",
            OptionStrategy::Calendar => "CALENDAR",
            OptionStrategy::Strangle => "STRANGLE",
            OptionStrategy::Straddle => "STRADDLE",
            OptionStrategy::Butterfly => "BUTTERFLY",
            OptionStrategy::Condor => "CONDOR",
            OptionStrategy::Diagonal => "DIAGONAL",
            OptionStrategy::Collar => "COLLAR",
            OptionStrategy::Roll => "ROLL",
        }
    }
}

/// All parameters for `GET /marketdata/v1/chains`.
#[derive(Debug, Clone, Default)]
pub struct OptionChainRequest {
    pub symbol: String,
    pub contract_type: Option<ContractType>,
    pub strike_count: Option<i32>,
    pub include_underlying_quote: Option<bool>,
    pub strategy: Option<OptionStrategy>,
    pub interval: Option<Money>,
    pub strike: Option<Money>,
    pub range: Option<OptionRange>,
    /// Format: "yyyy-MM-dd"
    pub from_date: Option<String>,
    /// Format: "yyyy-MM-dd"
    pub to_date: Option<String>,
    pub volatility: Option<Money>,
    pub underlying_price: Option<Money>,
    pub interest_rate: Option<Money>,
    pub days_to_expiration: Option<i32>,
    pub exp_month: Option<ExpirationMonth>,
    pub option_type: Option<OptionType>,
    pub entitlement: Option<String>,
}

/// A single option contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionContract {
    pub put_call: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub exchange_name: Option<String>,
    pub bid: Option<Money>,
    pub ask: Option<Money>,
    pub last: Option<Money>,
    pub mark: Option<Money>,
    pub bid_size: Option<i32>,
    pub ask_size: Option<i32>,
    pub bid_ask_size: Option<String>,
    pub last_size: Option<i32>,
    pub high_price: Option<Money>,
    pub low_price: Option<Money>,
    pub open_price: Option<Money>,
    pub close_price: Option<Money>,
    pub total_volume: Option<i64>,
    pub trade_date: Option<Timestamp>,
    pub trade_time_in_long: Option<Timestamp>,
    pub quote_time_in_long: Option<Timestamp>,
    pub net_change: Option<Money>,
    pub volatility: Option<Money>,
    pub delta: Option<Money>,
    pub gamma: Option<Money>,
    pub theta: Option<Money>,
    pub vega: Option<Money>,
    pub rho: Option<Money>,
    pub open_interest: Option<Money>,
    pub time_value: Option<Money>,
    pub theoretical_option_value: Option<Money>,
    pub theoretical_volatility: Option<Money>,
    pub strike_price: Option<Money>,
    pub expiration_date: Option<Timestamp>,
    pub days_to_expiration: Option<i32>,
    pub expiration_type: Option<String>,
    pub last_trading_day: Option<i64>,
    pub multiplier: Option<Money>,
    pub settlement_type: Option<String>,
    pub deliverable_note: Option<String>,
    pub is_index_option: Option<bool>,
    pub percent_change: Option<Money>,
    pub mark_change: Option<Money>,
    pub mark_percent_change: Option<Money>,
    pub in_the_money: Option<bool>,
    pub non_standard: Option<bool>,
    pub mini: Option<bool>,
    pub penny_pilot: Option<bool>,
}

/// Underlying instrument snapshot included in the option chain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnderlyingAsset {
    pub ask: Option<Money>,
    pub ask_size: Option<i32>,
    pub bid: Option<Money>,
    pub bid_size: Option<i32>,
    pub change: Option<Money>,
    pub close: Option<Money>,
    pub delayed: Option<bool>,
    pub description: Option<String>,
    pub exchange_name: Option<String>,
    pub fifty_two_week_high: Option<Money>,
    pub fifty_two_week_low: Option<Money>,
    pub high_price: Option<Money>,
    pub last: Option<Money>,
    pub low_price: Option<Money>,
    pub mark: Option<Money>,
    pub mark_change: Option<Money>,
    pub mark_percent_change: Option<Money>,
    pub open_price: Option<Money>,
    pub percent_change: Option<Money>,
    pub quote_time: Option<Timestamp>,
    pub symbol: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<Timestamp>,
}

/// Full option chain response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionChain {
    pub symbol: Option<String>,
    pub status: Option<String>,
    pub underlying: Option<UnderlyingAsset>,
    pub strategy: Option<String>,
    pub interval: Option<Money>,
    pub is_delayed: Option<bool>,
    pub is_index: Option<bool>,
    pub days_to_expiration: Option<Money>,
    pub interest_rate: Option<Money>,
    pub underlying_price: Option<Money>,
    pub volatility: Option<Money>,
    pub number_of_contracts: Option<i32>,
    /// Keyed by expiration date string → strike string → list of contracts.
    pub call_exp_date_map: Option<HashMap<String, HashMap<String, Vec<OptionContract>>>>,
    pub put_exp_date_map: Option<HashMap<String, HashMap<String, Vec<OptionContract>>>>,
}

/// An expiration available for an optionable underlying.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExpiration {
    pub expiration_date: Timestamp,
    pub days_to_expiration: i32,
    pub expiration_type: String,
    pub standard: bool,
    pub settlement_type: String,
    pub option_root: String,
}

/// Response from `GET /marketdata/v1/expirationchain`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExpirationChain {
    pub symbol: String,
    pub expiration_list: Vec<OptionExpiration>,
}
