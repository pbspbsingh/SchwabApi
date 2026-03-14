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
    HighPrice = 10,
    LowPrice = 11,
    ClosePrice = 12,
    ExchangeId = 13,
    Marginable = 14,
    Description = 15,
    LastId = 16,
    OpenPrice = 17,
    NetChange = 18,
    Week52High = 19,
    Week52Low = 20,
    PeRatio = 21,
    DividendAmount = 22,
    DividendYield = 23,
    Nav = 24,
    ExchangeName = 25,
    DividendDate = 26,
    IsRegularMarketQuote = 27,
    IsRegularMarketTrade = 28,
    RegularMarketLastPrice = 29,
    RegularMarketLastSize = 30,
    RegularMarketNetChange = 31,
    SecurityStatus = 32,
    Mark = 33,
    QuoteTimeMillis = 34,
    TradeTimeMillis = 35,
    RegularMarketTradeMillis = 36,
    BidTimeMillis = 37,
    AskTimeMillis = 38,
    AskMicId = 39,
    BidMicId = 40,
    LastMicId = 41,
    NetChangePercent = 42,
    RegularMarketChangePercent = 43,
    MarkChange = 44,
    MarkChangePercent = 45,
    HtbQuantity = 46,
    HtbRate = 47,
    HardToBorrow = 48,
    IsShortable = 49,
    PostMarketNetChange = 50,
    PostMarketNetChangePercent = 51,
}

impl LevelOneEquityField {
    /// Return all 52 fields.
    pub fn all() -> Vec<Self> {
        use LevelOneEquityField::*;
        vec![
            Symbol, BidPrice, AskPrice, LastPrice, BidSize, AskSize, AskId, BidId,
            TotalVolume, LastSize, HighPrice, LowPrice, ClosePrice, ExchangeId, Marginable,
            Description, LastId, OpenPrice, NetChange, Week52High, Week52Low, PeRatio,
            DividendAmount, DividendYield, Nav, ExchangeName, DividendDate,
            IsRegularMarketQuote, IsRegularMarketTrade, RegularMarketLastPrice,
            RegularMarketLastSize, RegularMarketNetChange, SecurityStatus, Mark,
            QuoteTimeMillis, TradeTimeMillis, RegularMarketTradeMillis, BidTimeMillis,
            AskTimeMillis, AskMicId, BidMicId, LastMicId, NetChangePercent,
            RegularMarketChangePercent, MarkChange, MarkChangePercent, HtbQuantity,
            HtbRate, HardToBorrow, IsShortable, PostMarketNetChange,
            PostMarketNetChangePercent,
        ]
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
    MoneyIntrinsicValue = 11,
    ExpirationYear = 12,
    Multiplier = 13,
    Digits = 14,
    OpenPrice = 15,
    BidSize = 16,
    AskSize = 17,
    LastSize = 18,
    NetChange = 19,
    StrikePrice = 20,
    ContractType = 21,
    Underlying = 22,
    ExpirationMonth = 23,
    Deliverables = 24,
    TimeValue = 25,
    ExpirationDay = 26,
    DaysToExpiration = 27,
    Delta = 28,
    Gamma = 29,
    Theta = 30,
    Vega = 31,
    Rho = 32,
    SecurityStatus = 33,
    TheoreticalOptionValue = 34,
    UnderlyingPrice = 35,
    UvExpirationType = 36,
    Mark = 37,
    QuoteTimeMillis = 38,
    TradeTimeMillis = 39,
    ExchangeId = 40,
    ExchangeName = 41,
    LastTradingDay = 42,
    SettlementType = 43,
    NetPercentChange = 44,
    MarkChange = 45,
    MarkPercentChange = 46,
    ImpliedYield = 47,
    IsPennyPilot = 48,
    OptionRoot = 49,
    Week52High = 50,
    Week52Low = 51,
    IndicativeAskingPrice = 52,
    IndicativeBidPrice = 53,
    IndicativeQuoteTime = 54,
    ExerciseType = 55,
}

impl LevelOneOptionField {
    /// Return all 56 fields.
    pub fn all() -> Vec<Self> {
        use LevelOneOptionField::*;
        vec![
            Symbol, Description, BidPrice, AskPrice, LastPrice, HighPrice, LowPrice,
            ClosePrice, TotalVolume, OpenInterest, Volatility, MoneyIntrinsicValue,
            ExpirationYear, Multiplier, Digits, OpenPrice, BidSize, AskSize, LastSize,
            NetChange, StrikePrice, ContractType, Underlying, ExpirationMonth, Deliverables,
            TimeValue, ExpirationDay, DaysToExpiration, Delta, Gamma, Theta, Vega, Rho,
            SecurityStatus, TheoreticalOptionValue, UnderlyingPrice, UvExpirationType, Mark,
            QuoteTimeMillis, TradeTimeMillis, ExchangeId, ExchangeName, LastTradingDay,
            SettlementType, NetPercentChange, MarkChange, MarkPercentChange, ImpliedYield,
            IsPennyPilot, OptionRoot, Week52High, Week52Low, IndicativeAskingPrice,
            IndicativeBidPrice, IndicativeQuoteTime, ExerciseType,
        ]
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
    BidId = 6,
    AskId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTimeMillis = 10,
    TradeTimeMillis = 11,
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
    AskTimeMillis = 37,
    BidTimeMillis = 38,
    QuotedInSession = 39,
    SettlementDate = 40,
}

impl LevelOneFuturesField {
    /// Return all 41 fields.
    pub fn all() -> Vec<Self> {
        use LevelOneFuturesField::*;
        vec![
            Symbol, BidPrice, AskPrice, LastPrice, BidSize, AskSize, BidId, AskId,
            TotalVolume, LastSize, QuoteTimeMillis, TradeTimeMillis, HighPrice, LowPrice,
            ClosePrice, ExchangeId, Description, LastId, OpenPrice, NetChange,
            FuturePercentChange, ExchangeName, SecurityStatus, OpenInterest, Mark, Tick,
            TickAmount, Product, FuturePriceFormat, FutureTradingHours, FutureIsTradable,
            FutureMultiplier, FutureIsActive, FutureSettlementPrice, FutureActiveSymbol,
            FutureExpirationDate, ExpirationStyle, AskTimeMillis, BidTimeMillis,
            QuotedInSession, SettlementDate,
        ]
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
    QuoteTimeMillis = 8,
    TradeTimeMillis = 9,
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
        use LevelOneForexField::*;
        vec![
            Symbol, BidPrice, AskPrice, LastPrice, BidSize, AskSize, TotalVolume, LastSize,
            QuoteTimeMillis, TradeTimeMillis, HighPrice, LowPrice, ClosePrice, ExchangeId,
            Description, OpenPrice, NetChange, PercentChange, ExchangeName, Digits,
            SecurityStatus, Tick, TickAmount, Product, TradingHours, IsTradable, MarketMaker,
            Week52High, Week52Low, Mark,
        ]
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
    BidId = 6,
    AskId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTimeMillis = 10,
    TradeTimeMillis = 11,
    HighPrice = 12,
    LowPrice = 13,
    ClosePrice = 14,
    LastId = 15,
    Description = 16,
    OpenPrice = 17,
    OpenInterest = 18,
    Mark = 19,
    Tick = 20,
    TickAmount = 21,
    FutureMultiplier = 22,
    FutureSettlementPrice = 23,
    UnderlyingSymbol = 24,
    StrikePrice = 25,
    FutureExpirationDate = 26,
    ExpirationStyle = 27,
    ContractType = 28,
    SecurityStatus = 29,
    ExchangeId = 30,
    ExchangeName = 31,
}

impl LevelOneFuturesOptionField {
    /// Return all 32 fields.
    pub fn all() -> Vec<Self> {
        use LevelOneFuturesOptionField::*;
        vec![
            Symbol, BidPrice, AskPrice, LastPrice, BidSize, AskSize, BidId, AskId,
            TotalVolume, LastSize, QuoteTimeMillis, TradeTimeMillis, HighPrice, LowPrice,
            ClosePrice, LastId, Description, OpenPrice, OpenInterest, Mark, Tick, TickAmount,
            FutureMultiplier, FutureSettlementPrice, UnderlyingSymbol, StrikePrice,
            FutureExpirationDate, ExpirationStyle, ContractType, SecurityStatus,
            ExchangeId, ExchangeName,
        ]
    }
}

// ── ChartEquityField ──────────────────────────────────────────────────────────

/// Subscribable fields for the `CHART_EQUITY` service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChartEquityField {
    Symbol = 0,
    Sequence = 1,
    OpenPrice = 2,
    HighPrice = 3,
    LowPrice = 4,
    ClosePrice = 5,
    Volume = 6,
    ChartTime = 7,
    ChartDay = 8,
}

impl ChartEquityField {
    /// Return all 9 fields.
    pub fn all() -> Vec<Self> {
        use ChartEquityField::*;
        vec![Symbol, Sequence, OpenPrice, HighPrice, LowPrice, ClosePrice, Volume, ChartTime, ChartDay]
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
        use ChartFuturesField::*;
        vec![Symbol, ChartTime, OpenPrice, HighPrice, LowPrice, ClosePrice, Volume]
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
