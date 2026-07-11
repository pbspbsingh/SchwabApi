//! Instrument search and lookup models.

use serde::{Deserialize, Serialize};

use crate::types::Money;

/// Instrument search projection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    /// Symbol starts-with match.
    SymbolSearch,
    /// Symbol regex match.
    SymbolRegex,
    /// Description contains match.
    DescSearch,
    /// Description regex match.
    DescRegex,
    /// Return the instrument directly.
    SearchFundamental,
    /// Full instrument details.
    Full,
}

impl Projection {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Projection::SymbolSearch => "symbol-search",
            Projection::SymbolRegex => "symbol-regex",
            Projection::DescSearch => "desc-search",
            Projection::DescRegex => "desc-regex",
            Projection::SearchFundamental => "fundamental",
            Projection::Full => "full",
        }
    }
}

/// Instrument asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetType {
    Bond,
    Equity,
    Etf,
    Extended,
    ForexCurrency,
    Future,
    FutureOption,
    Fundamental,
    Index,
    Indicator,
    MutualFund,
    Option,
    Unknown,
}

/// Fundamental data for an instrument.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentFundamental {
    pub symbol: Option<String>,
    pub high52: Option<Money>,
    pub low52: Option<Money>,
    pub dividend_amount: Option<Money>,
    pub dividend_yield: Option<Money>,
    pub dividend_date: Option<String>,
    pub pe_ratio: Option<Money>,
    pub peg_ratio: Option<Money>,
    pub pb_ratio: Option<Money>,
    pub pr_ratio: Option<Money>,
    pub pcf_ratio: Option<Money>,
    pub gross_margin_ttm: Option<Money>,
    pub gross_margin_mrq: Option<Money>,
    pub net_profit_margin_ttm: Option<Money>,
    pub net_profit_margin_mrq: Option<Money>,
    pub operating_margin_ttm: Option<Money>,
    pub operating_margin_mrq: Option<Money>,
    pub return_on_equity: Option<Money>,
    pub return_on_assets: Option<Money>,
    pub return_on_investment: Option<Money>,
    pub quick_ratio: Option<Money>,
    pub current_ratio: Option<Money>,
    pub interest_coverage: Option<Money>,
    pub total_debt_to_capital: Option<Money>,
    pub lt_debt_to_equity: Option<Money>,
    pub total_debt_to_equity: Option<Money>,
    pub eps_ttm: Option<Money>,
    pub eps_change_percent_ttm: Option<Money>,
    pub eps_change_year: Option<Money>,
    pub eps_change: Option<Money>,
    pub rev_change_year: Option<Money>,
    pub rev_change_ttm: Option<Money>,
    pub rev_change_in: Option<Money>,
    pub shares_outstanding: Option<Money>,
    pub market_cap_float: Option<Money>,
    pub market_cap: Option<Money>,
    pub book_value_per_share: Option<Money>,
    pub short_int_to_float: Option<Money>,
    pub short_int_day_to_cover: Option<Money>,
    pub div_growth_rate3_year: Option<Money>,
    pub dividend_pay_amount: Option<Money>,
    pub dividend_pay_date: Option<String>,
    pub beta: Option<Money>,
    pub vol1_day_avg: Option<Money>,
    pub vol10_day_avg: Option<Money>,
    pub vol3_month_avg: Option<Money>,
}

/// A single financial instrument.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Instrument {
    pub asset_type: Option<AssetType>,
    pub cusip: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub fundamental: Option<InstrumentFundamental>,
    pub instrument_info: Option<serde_json::Value>,
    pub bond_factor: Option<Money>,
    pub bond_multiplier: Option<Money>,
    pub bond_price: Option<Money>,
}

/// Response wrapper from `GET /marketdata/v1/instruments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentsResponse {
    pub instruments: Option<Vec<Instrument>>,
}
