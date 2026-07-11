//! Account-related models.

use serde::{Deserialize, Serialize};

use crate::types::Money;

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
    pub cash_balance: Option<Money>,
    pub accrued_interest: Option<Money>,
    pub available_funds: Option<Money>,
    pub available_funds_non_marginable_trade: Option<Money>,
    pub buying_power: Option<Money>,
    pub buying_power_non_marginable_trade: Option<Money>,
    pub day_trading_buying_power: Option<Money>,
    pub day_trading_buying_power_call: Option<Money>,
    pub equity: Option<Money>,
    pub equity_percentage: Option<Money>,
    pub long_margin_value: Option<Money>,
    pub maintenance_call: Option<Money>,
    pub maintenance_requirement: Option<Money>,
    pub margin: Option<Money>,
    pub margin_equity: Option<Money>,
    pub money_market_fund: Option<Money>,
    pub mutual_fund_value: Option<Money>,
    pub reg_t_call: Option<Money>,
    pub short_margin_value: Option<Money>,
    pub short_option_market_value: Option<Money>,
    pub short_stock_value: Option<Money>,
    pub total_cash: Option<Money>,
    pub is_in_call: Option<bool>,
    pub unsettled_cash: Option<Money>,
    pub pending_deposits: Option<Money>,
    pub margin_balance: Option<Money>,
    pub short_balance: Option<Money>,
    pub account_value: Option<Money>,
}

/// A single position held in an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub short_quantity: Option<Money>,
    pub average_price: Option<Money>,
    pub current_day_profit_loss: Option<Money>,
    pub current_day_profit_loss_percentage: Option<Money>,
    pub long_quantity: Option<Money>,
    pub settled_long_quantity: Option<Money>,
    pub settled_short_quantity: Option<Money>,
    pub agged_long_quantity: Option<Money>,
    pub agged_short_quantity: Option<Money>,
    pub instrument: Option<PositionInstrument>,
    pub market_value: Option<Money>,
    pub maintenance_requirement: Option<Money>,
    pub average_long_price: Option<Money>,
    pub average_short_price: Option<Money>,
    pub tax_lot_average_long_price: Option<Money>,
    pub tax_lot_average_short_price: Option<Money>,
    pub long_open_profit_loss: Option<Money>,
    pub short_open_profit_loss: Option<Money>,
    pub previous_session_long_quantity: Option<Money>,
    pub previous_session_short_quantity: Option<Money>,
    pub current_day_cost: Option<Money>,
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
    pub net_change: Option<Money>,
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
