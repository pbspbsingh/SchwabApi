//! Market mover (top gainers/losers) models.

use serde::{Deserialize, Serialize};

/// The index to query movers for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Index {
    /// NYSE ARCA Major Market Index.
    DowJones,
    /// NASDAQ Composite.
    Nasdaq,
    /// S&P 500.
    Sp500,
    /// NYSE Composite.
    NyseComposite,
    /// Russell 2000.
    R2000,
    /// Dow Jones 30.
    Dji,
}

impl Index {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Index::DowJones => "$DJI",
            Index::Nasdaq => "$COMPX",
            Index::Sp500 => "$SPX",
            Index::NyseComposite => "$NYA",
            Index::R2000 => "RUT",
            Index::Dji => "$DJI",
        }
    }
}

/// Sort order for movers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Largest percentage change (up or down).
    Volume,
    /// Largest trades.
    Trades,
    /// Percentage change ascending.
    PercentChangeUp,
    /// Percentage change descending.
    PercentChangeDown,
}

impl SortOrder {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            SortOrder::Volume => "VOLUME",
            SortOrder::Trades => "TRADES",
            SortOrder::PercentChangeUp => "PERCENT_CHANGE_UP",
            SortOrder::PercentChangeDown => "PERCENT_CHANGE_DOWN",
        }
    }
}

/// Frequency of the mover calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoverFrequency {
    Zero,
    One,
    Five,
    Ten,
    Thirty,
    Sixty,
}

impl MoverFrequency {
    pub(crate) fn as_u32(self) -> u32 {
        match self {
            MoverFrequency::Zero => 0,
            MoverFrequency::One => 1,
            MoverFrequency::Five => 5,
            MoverFrequency::Ten => 10,
            MoverFrequency::Thirty => 30,
            MoverFrequency::Sixty => 60,
        }
    }
}

/// A single mover entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mover {
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub direction: Option<String>,
    pub change: Option<f64>,
    pub last: Option<f64>,
    pub total_volume: Option<i64>,
    pub volume: Option<i64>,
    pub net_change: Option<f64>,
    pub market_share: Option<f64>,
    pub trades: Option<i64>,
    pub net_percent_change: Option<f64>,
}

/// Response wrapper for movers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoversResponse {
    pub screeners: Option<Vec<Mover>>,
}
