//! Transaction models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque transaction identifier.
pub type TransactionId = i64;

/// Transaction type filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Trade,
    ReceiveAndDeliver,
    DividendOrInterest,
    AchReceipt,
    AchDisbursement,
    CashReceipt,
    CashDisbursement,
    ElectronicFund,
    WireOut,
    WireIn,
    Journal,
    Memorandum,
    MarginCall,
    MoneyMarket,
    SmaAdjustment,
}

impl TransactionType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            TransactionType::Trade => "TRADE",
            TransactionType::ReceiveAndDeliver => "RECEIVE_AND_DELIVER",
            TransactionType::DividendOrInterest => "DIVIDEND_OR_INTEREST",
            TransactionType::AchReceipt => "ACH_RECEIPT",
            TransactionType::AchDisbursement => "ACH_DISBURSEMENT",
            TransactionType::CashReceipt => "CASH_RECEIPT",
            TransactionType::CashDisbursement => "CASH_DISBURSEMENT",
            TransactionType::ElectronicFund => "ELECTRONIC_FUND",
            TransactionType::WireOut => "WIRE_OUT",
            TransactionType::WireIn => "WIRE_IN",
            TransactionType::Journal => "JOURNAL",
            TransactionType::Memorandum => "MEMORANDUM",
            TransactionType::MarginCall => "MARGIN_CALL",
            TransactionType::MoneyMarket => "MONEY_MARKET",
            TransactionType::SmaAdjustment => "SMA_ADJUSTMENT",
        }
    }
}

/// Parameters for `GET /trader/v1/accounts/{hash}/transactions`.
#[derive(Debug, Clone, Default)]
pub struct GetTransactionsRequest {
    pub transaction_type: Option<TransactionType>,
    pub symbol: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Instrument details within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInstrument {
    pub asset_type: Option<String>,
    pub cusip: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<f64>,
    pub put_call: Option<String>,
    pub underlying_symbol: Option<String>,
    pub option_expiration_date: Option<DateTime<Utc>>,
    pub option_strike_price: Option<f64>,
    pub type_: Option<String>,
}

/// A single transaction on an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub activity_id: Option<i64>,
    pub time: Option<DateTime<Utc>>,
    pub user: Option<serde_json::Value>,
    pub description: Option<String>,
    pub account_number: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub sub_account: Option<String>,
    pub trade_date: Option<DateTime<Utc>>,
    pub settlement_date: Option<DateTime<Utc>>,
    pub position_id: Option<i64>,
    pub order_id: Option<i64>,
    pub net_amount: Option<f64>,
    pub activity_type: Option<String>,
    pub transfer_items: Option<Vec<TransferItem>>,
}

/// An individual leg of a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferItem {
    pub instrument: Option<TransactionInstrument>,
    pub amount: Option<f64>,
    pub cost: Option<f64>,
    pub price: Option<f64>,
    pub fee_type: Option<String>,
    pub position_effect: Option<String>,
}
