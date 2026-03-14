//! Typed field enums for each streaming service.
//!
//! Each enum variant maps to the numeric field ID sent to the server
//! in the `fields` subscription parameter. Use [`LevelOneEquityField::all()`]
//! (or the equivalent for another service) to subscribe to every field.

// ── LevelOneEquityField ───────────────────────────────────────────────────────

/// Subscribable fields for the `LEVELONE_EQUITIES` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneEquityField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    AskId = 6,
    BidId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTime = 10,
    TradeTime = 11,
    HighPrice = 12,
    LowPrice = 13,
    BidTick = 14,
    ClosePrice = 15,
    ExchangeId = 16,
    Marginable = 17,
    Shortable = 18,
    IslandBidPrice = 19,
    IslandAskPrice = 20,
    IslandVolume = 21,
    QuoteDay = 22,
    TradeDay = 23,
    Volatility = 24,
    Description = 25,
    LastId = 26,
    Digits = 27,
    OpenPrice = 28,
    NetChange = 29,
    Week52High = 30,
    Week52Low = 31,
    PeRatio = 32,
    DividendAmount = 33,
    DividendYield = 34,
    IslandBidSize = 35,
    IslandAskSize = 36,
    Nav = 37,
    FundPrice = 38,
    ExchangeName = 39,
    DividendDate = 40,
    IsRegularMarketQuote = 41,
    IsRegularMarketTrade = 42,
    RegularMarketLastPrice = 43,
    RegularMarketLastSize = 44,
    RegularMarketTradeTime = 45,
    RegularMarketTradeDay = 46,
    RegularMarketNetChange = 47,
    SecurityStatus = 48,
    Mark = 49,
    QuoteTimeMillis = 50,
    TradeTimeMillis = 51,
}

impl LevelOneEquityField {
    /// Return all 52 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=51)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── LevelOneOptionField ───────────────────────────────────────────────────────

/// Subscribable fields for the `LEVELONE_OPTIONS` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneOptionField {
    Symbol = 0,
    Description = 1,
    BidPrice = 2,
    AskPrice = 3,
    LastPrice = 4,
    HighPrice = 5,
    LowPrice = 6,
    ClosePrice = 7,
    TotalVolume = 8,
    OpenInterest = 9,
    Volatility = 10,
    QuoteTime = 11,
    TradeTime = 12,
    MoneyIntrinsicValue = 13,
    QuoteDay = 14,
    TradeDay = 15,
    ExpirationYear = 16,
    Multiplier = 17,
    Digits = 18,
    OpenPrice = 19,
    BidSize = 20,
    AskSize = 21,
    LastSize = 22,
    NetChange = 23,
    StrikePrice = 24,
    ContractType = 25,
    Underlying = 26,
    ExpirationMonth = 27,
    Deliverables = 28,
    TimeValue = 29,
    ExpirationDay = 30,
    DaysToExpiration = 31,
    Delta = 32,
    Gamma = 33,
    Theta = 34,
    Vega = 35,
    Rho = 36,
    SecurityStatus = 37,
    TheoreticalOptionValue = 38,
    UnderlyingPrice = 39,
    UvExpirationType = 40,
    Mark = 41,
    QuoteTimeMillis = 42,
    TradeTimeMillis = 43,
    ExchangeId = 44,
    ExchangeName = 45,
    LastTradingDay = 46,
    SettlementType = 47,
    NetPercentChange = 48,
    MarkChange = 49,
    MarkPercentChange = 50,
    ImpliedYield = 51,
    IsPennyPilot = 52,
    OptionRoot = 53,
    Week52High = 54,
    Week52Low = 55,
}

impl LevelOneOptionField {
    /// Return all 56 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=55)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── LevelOneFuturesField ──────────────────────────────────────────────────────

/// Subscribable fields for the `LEVELONE_FUTURES` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneFuturesField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    AskId = 6,
    BidId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTime = 10,
    TradeTime = 11,
    HighPrice = 12,
    LowPrice = 13,
    ClosePrice = 14,
    ExchangeId = 15,
    Description = 16,
    LastId = 17,
    OpenPrice = 18,
    NetChange = 19,
    FuturePercentChange = 20,
    ExchangeName = 21,
    SecurityStatus = 22,
    OpenInterest = 23,
    Mark = 24,
    Tick = 25,
    TickAmount = 26,
    Product = 27,
    FuturePriceFormat = 28,
    FutureTradingHours = 29,
    FutureIsTradable = 30,
    FutureMultiplier = 31,
    FutureIsActive = 32,
    FutureSettlementPrice = 33,
    FutureActiveSymbol = 34,
    FutureExpirationDate = 35,
    ExpirationStyle = 36,
    AskTime = 37,
    BidTime = 38,
    QuotedInSession = 39,
    SettlementDate = 40,
}

impl LevelOneFuturesField {
    /// Return all 41 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=40)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── LevelOneForexField ────────────────────────────────────────────────────────

/// Subscribable fields for the `LEVELONE_FOREX` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneForexField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    TotalVolume = 6,
    LastSize = 7,
    QuoteTime = 8,
    TradeTime = 9,
    HighPrice = 10,
    LowPrice = 11,
    ClosePrice = 12,
    ExchangeId = 13,
    Description = 14,
    OpenPrice = 15,
    NetChange = 16,
    PercentChange = 17,
    ExchangeName = 18,
    Digits = 19,
    SecurityStatus = 20,
    Tick = 21,
    TickAmount = 22,
    Product = 23,
    TradingHours = 24,
    IsTradable = 25,
    MarketMaker = 26,
    Week52High = 27,
    Week52Low = 28,
    Mark = 29,
}

impl LevelOneForexField {
    /// Return all 30 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=29)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── LevelOneFuturesOptionField ────────────────────────────────────────────────

/// Subscribable fields for the `LEVELONE_FUTURES_OPTIONS` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LevelOneFuturesOptionField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    AskId = 6,
    BidId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTime = 10,
    TradeTime = 11,
    HighPrice = 12,
    LowPrice = 13,
    ClosePrice = 14,
    ExchangeId = 15,
    Description = 16,
    LastId = 17,
    OpenPrice = 18,
    NetChange = 19,
    FuturePercentChange = 20,
    ExchangeName = 21,
    SecurityStatus = 22,
    OpenInterest = 23,
    Mark = 24,
    Tick = 25,
    TickAmount = 26,
    Product = 27,
    FuturePriceFormat = 28,
    FutureTradingHours = 29,
    FutureIsTradable = 30,
    FutureMultiplier = 31,
}

impl LevelOneFuturesOptionField {
    /// Return all 32 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=31)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── ChartEquityField ──────────────────────────────────────────────────────────

/// Subscribable fields for the `CHART_EQUITY` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChartEquityField {
    Symbol = 0,
    OpenPrice = 1,
    HighPrice = 2,
    LowPrice = 3,
    ClosePrice = 4,
    Volume = 5,
    Sequence = 6,
    ChartTime = 7,
    ChartDay = 8,
}

impl ChartEquityField {
    /// Return all 9 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=8)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── ChartFuturesField ─────────────────────────────────────────────────────────

/// Subscribable fields for the `CHART_FUTURES` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChartFuturesField {
    Symbol = 0,
    ChartTime = 1,
    OpenPrice = 2,
    HighPrice = 3,
    LowPrice = 4,
    ClosePrice = 5,
    Volume = 6,
}

impl ChartFuturesField {
    /// Return all 7 fields.
    pub fn all() -> Vec<Self> {
        (0u32..=6)
            .map(|n| unsafe { std::mem::transmute(n) })
            .collect()
    }
}

// ── BookField ─────────────────────────────────────────────────────────────────

/// Subscribable fields for the book services
/// (`NYSE_BOOK`, `NASDAQ_BOOK`, `OPTIONS_BOOK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BookField {
    Symbol = 0,
    BookTime = 1,
    Bids = 2,
    Asks = 3,
}

impl BookField {
    /// Return all 4 fields.
    pub fn all() -> Vec<Self> {
        vec![
            BookField::Symbol,
            BookField::BookTime,
            BookField::Bids,
            BookField::Asks,
        ]
    }
}

// ── ScreenerField ─────────────────────────────────────────────────────────────

/// Subscribable fields for the `SCREENER_EQUITY` and `SCREENER_OPTION` services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ScreenerField {
    Symbol = 0,
    Timestamp = 1,
    SortField = 2,
    Frequency = 3,
    Items = 4,
}

impl ScreenerField {
    /// Return all 5 fields.
    pub fn all() -> Vec<Self> {
        vec![
            ScreenerField::Symbol,
            ScreenerField::Timestamp,
            ScreenerField::SortField,
            ScreenerField::Frequency,
            ScreenerField::Items,
        ]
    }
}

// ── AccountActivityField ──────────────────────────────────────────────────────

/// Subscribable fields for the `ACCT_ACTIVITY` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AccountActivityField {
    SubscriptionKey = 0,
    AccountNumber = 1,
    MessageType = 2,
    MessageData = 3,
}

impl AccountActivityField {
    /// Return all 4 fields.
    pub fn all() -> Vec<Self> {
        vec![
            AccountActivityField::SubscriptionKey,
            AccountActivityField::AccountNumber,
            AccountActivityField::MessageType,
            AccountActivityField::MessageData,
        ]
    }
}
