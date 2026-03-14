//! Order models for placing, modifying, and querying orders.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque order identifier.
pub type OrderId = i64;

/// The primary order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    #[default]
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
    Cabinet,
    NonMarketable,
    MarketOnClose,
    Exercise,
    TrailingStopLimit,
    NetDebit,
    NetCredit,
    NetZero,
    LimitOnClose,
    Unknown,
}

/// Trading session in which the order is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Session {
    #[default]
    Normal,
    Am,
    Pm,
    Seamless,
}

/// Time-in-force for the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Duration {
    #[default]
    Day,
    GoodTillCancel,
    FillOrKill,
    ImmediateOrCancel,
    EndOfWeek,
    EndOfMonth,
    NextEndOfMonth,
    Unknown,
}

/// High-level strategy classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStrategyType {
    #[default]
    Single,
    Cancel,
    Recall,
    Pair,
    Flatten,
    TwoDaySwap,
    BlastAll,
    Oco,
    Trigger,
}

/// Buy/sell direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Instruction {
    #[default]
    Buy,
    Sell,
    BuyToCover,
    SellShort,
    BuyToOpen,
    BuyToClose,
    SellToOpen,
    SellToClose,
    Exchange,
    SellShortExempt,
}

/// Position effect for option orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionEffect {
    Opening,
    Closing,
    Automatic,
}

/// Current status of an order on the exchange/OMS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    AwaitingParentOrder,
    AwaitingCondition,
    AwaitingStopCondition,
    AwaitingManualReview,
    Accepted,
    AwaitingUrOut,
    PendingActivation,
    Queued,
    Working,
    Rejected,
    PendingCancel,
    Canceled,
    PendingReplace,
    Replaced,
    Filled,
    Expired,
    New,
    AwaitingReleaseTime,
    PendingAcknowledgement,
    PendingRecall,
    Unknown,
}

/// How the order price is specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceLinkBasis {
    Manual,
    Base,
    Trigger,
    Last,
    Bid,
    Ask,
    AskBid,
    Mark,
    Average,
}

/// Offset type for price-linked orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceLinkType {
    Value,
    Percent,
    Tick,
}

/// Stop price linking type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopType {
    Standard,
    Bid,
    Ask,
    Last,
    Mark,
}

/// Tax lot method for the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxLotMethod {
    Fifo,
    Lifo,
    HighCost,
    LowCost,
    AverageCost,
    SpecificLot,
    LossHarvester,
}

/// Request envelope for the `GET /accounts/{hash}/orders` and `GET /orders` endpoints.
#[derive(Debug, Clone, Default)]
pub struct GetOrdersRequest {
    pub from_entered_time: Option<DateTime<Utc>>,
    pub to_entered_time: Option<DateTime<Utc>>,
    pub max_results: Option<i32>,
    pub status: Option<OrderStatus>,
}

/// An order leg describing a single instrument/instruction pair.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrderLeg {
    pub order_leg_type: Option<String>,
    pub leg_id: Option<i64>,
    pub instrument: Option<OrderInstrument>,
    pub instruction: Option<Instruction>,
    pub position_effect: Option<PositionEffect>,
    pub quantity: Option<f64>,
    pub quantity_type: Option<String>,
    pub div_cap_gains: Option<String>,
    pub to_symbol: Option<String>,
}

/// Instrument referenced in an order leg.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrderInstrument {
    pub asset_type: Option<String>,
    pub cusip: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<f64>,
    pub put_call: Option<String>,
    pub underlying_symbol: Option<String>,
    pub option_multiplier: Option<f64>,
    pub option_deliverables: Option<Vec<serde_json::Value>>,
}

/// An activity record attached to an order (fills, cancellations, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderActivity {
    pub activity_type: Option<String>,
    pub activity_id: Option<i64>,
    pub execution_type: Option<String>,
    pub quantity: Option<f64>,
    pub order_remaining_quantity: Option<f64>,
    pub execution_legs: Option<Vec<ExecutionLeg>>,
}

/// A single filled quantity at a specific price.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLeg {
    pub leg_id: Option<i64>,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub mismarked_quantity: Option<f64>,
    pub instrument_id: Option<i64>,
    pub time: Option<DateTime<Utc>>,
}

/// A complete order, used for both reading and writing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Present on read; omit when placing a new order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<OrderId>,
    pub order_type: OrderType,
    pub session: Session,
    pub duration: Duration,
    pub order_strategy_type: OrderStrategyType,
    pub order_leg_collection: Vec<OrderLeg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entered_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complex_order_strategy_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_link_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price_link_basis: Option<PriceLinkBasis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price_link_type: Option<PriceLinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_type: Option<StopType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_link_basis: Option<PriceLinkBasis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_link_type: Option<PriceLinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lot_method: Option<TaxLotMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divleg_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releasing_order_id: Option<OrderId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_order_strategies: Option<Vec<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_activity_collection: Option<Vec<OrderActivity>>,
}
