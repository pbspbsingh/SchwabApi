//! Instrument search and lookup models.

use serde::{Deserialize, Serialize};

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
    pub high52: Option<f64>,
    pub low52: Option<f64>,
    pub dividend_amount: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub dividend_date: Option<String>,
    pub pe_ratio: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub pr_ratio: Option<f64>,
    pub pcf_ratio: Option<f64>,
    pub gross_margin_ttm: Option<f64>,
    pub gross_margin_mrq: Option<f64>,
    pub net_profit_margin_ttm: Option<f64>,
    pub net_profit_margin_mrq: Option<f64>,
    pub operating_margin_ttm: Option<f64>,
    pub operating_margin_mrq: Option<f64>,
    pub return_on_equity: Option<f64>,
    pub return_on_assets: Option<f64>,
    pub return_on_investment: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub current_ratio: Option<f64>,
    pub interest_coverage: Option<f64>,
    pub total_debt_to_capital: Option<f64>,
    pub lt_debt_to_equity: Option<f64>,
    pub total_debt_to_equity: Option<f64>,
    pub eps_ttm: Option<f64>,
    pub eps_change_percent_ttm: Option<f64>,
    pub eps_change_year: Option<f64>,
    pub eps_change: Option<f64>,
    pub rev_change_year: Option<f64>,
    pub rev_change_ttm: Option<f64>,
    pub rev_change_in: Option<f64>,
    pub shares_outstanding: Option<f64>,
    pub market_cap_float: Option<f64>,
    pub market_cap: Option<f64>,
    pub book_value_per_share: Option<f64>,
    pub short_int_to_float: Option<f64>,
    pub short_int_day_to_cover: Option<f64>,
    pub div_growth_rate3_year: Option<f64>,
    pub dividend_pay_amount: Option<f64>,
    pub dividend_pay_date: Option<String>,
    pub beta: Option<f64>,
    pub vol1_day_avg: Option<f64>,
    pub vol10_day_avg: Option<f64>,
    pub vol3_month_avg: Option<f64>,
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
    pub bond_factor: Option<f64>,
    pub bond_multiplier: Option<f64>,
    pub bond_price: Option<f64>,
}

/// Response wrapper from `GET /marketdata/v1/instruments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentsResponse {
    pub instruments: Option<Vec<Instrument>>,
}
