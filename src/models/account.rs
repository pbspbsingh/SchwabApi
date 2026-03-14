//! Account-related models.

use serde::{Deserialize, Serialize};

/// Maps a human-readable account number to the hash value used in API calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountNumber {
    /// The display account number (e.g. "****1234").
    pub account_number: String,
    /// The opaque hash to pass to account-specific endpoints.
    pub hash_value: String,
}

/// Optional fields to include when fetching account data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountFields {
    /// Include current positions.
    Positions,
}

impl AccountFields {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AccountFields::Positions => "positions",
        }
    }
}

/// Top-level account response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Type discriminator ("securitiesAccount").
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub securities_account: Option<SecuritiesAccount>,
}

/// A brokerage securities account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritiesAccount {
    /// Account type string (e.g. "MARGIN", "CASH").
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    /// The hashed account ID used in endpoints.
    pub account_number: Option<String>,
    /// Round-trip count for day trading.
    pub round_trips: Option<i32>,
    pub is_day_trader: Option<bool>,
    pub is_closing_only_restricted: Option<bool>,
    pub pfcb_flag: Option<bool>,
    /// Current balances.
    pub current_balances: Option<AccountBalance>,
    /// Initial balances at market open.
    pub initial_balances: Option<AccountBalance>,
    /// Projected balances.
    pub projected_balances: Option<AccountBalance>,
    /// Open positions (present when `fields=positions` is requested).
    pub positions: Option<Vec<Position>>,
}

/// Balance information for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub cash_balance: Option<f64>,
    pub accrued_interest: Option<f64>,
    pub available_funds: Option<f64>,
    pub available_funds_non_marginable_trade: Option<f64>,
    pub buying_power: Option<f64>,
    pub buying_power_non_marginable_trade: Option<f64>,
    pub day_trading_buying_power: Option<f64>,
    pub day_trading_buying_power_call: Option<f64>,
    pub equity: Option<f64>,
    pub equity_percentage: Option<f64>,
    pub long_margin_value: Option<f64>,
    pub maintenance_call: Option<f64>,
    pub maintenance_requirement: Option<f64>,
    pub margin: Option<f64>,
    pub margin_equity: Option<f64>,
    pub money_market_fund: Option<f64>,
    pub mutual_fund_value: Option<f64>,
    pub reg_t_call: Option<f64>,
    pub short_margin_value: Option<f64>,
    pub short_option_market_value: Option<f64>,
    pub short_stock_value: Option<f64>,
    pub total_cash: Option<f64>,
    pub is_in_call: Option<bool>,
    pub unsettled_cash: Option<f64>,
    pub pending_deposits: Option<f64>,
    pub margin_balance: Option<f64>,
    pub short_balance: Option<f64>,
    pub account_value: Option<f64>,
}

/// A single position held in an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub short_quantity: Option<f64>,
    pub average_price: Option<f64>,
    pub current_day_profit_loss: Option<f64>,
    pub current_day_profit_loss_percentage: Option<f64>,
    pub long_quantity: Option<f64>,
    pub settled_long_quantity: Option<f64>,
    pub settled_short_quantity: Option<f64>,
    pub agged_long_quantity: Option<f64>,
    pub agged_short_quantity: Option<f64>,
    pub instrument: Option<PositionInstrument>,
    pub market_value: Option<f64>,
    pub maintenance_requirement: Option<f64>,
    pub average_long_price: Option<f64>,
    pub average_short_price: Option<f64>,
    pub tax_lot_average_long_price: Option<f64>,
    pub tax_lot_average_short_price: Option<f64>,
    pub long_open_profit_loss: Option<f64>,
    pub short_open_profit_loss: Option<f64>,
    pub previous_session_long_quantity: Option<f64>,
    pub previous_session_short_quantity: Option<f64>,
    pub current_day_cost: Option<f64>,
}

/// Instrument details within a position.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInstrument {
    pub asset_type: Option<String>,
    pub cusip: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<f64>,
}

/// User preferences, including streaming connection details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    /// One entry per authorized account; the first is used for streaming.
    pub streamer_info: Vec<StreamerInfo>,
    pub offers: Option<Vec<serde_json::Value>>,
}

/// Streaming-server connection credentials.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamerInfo {
    /// WSS endpoint URL.
    pub streamer_socket_url: String,
    pub schwab_client_customer_id: String,
    pub schwab_client_correl_id: String,
    pub schwab_client_channel: String,
    pub schwab_client_function_id: String,
}
