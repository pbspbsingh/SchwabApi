//! REST API client for the Schwab brokerage platform.
//!
//! # Usage
//! ```rust,no_run
//! use std::sync::Arc;
//! use schwab_api::{SchwabClient, TokenManager, OAuthConfig, TokenSet};
//!
//! # async fn example() -> schwab_api::Result<()> {
//! # let config = OAuthConfig { app_key: String::new(), app_secret: String::new(), redirect_uri: String::new() };
//! # let tokens_set: TokenSet = unimplemented!();
//! let tokens = TokenManager::new(config, tokens_set);
//! let client = SchwabClient::new(Arc::clone(&tokens));
//! let accounts = client.get_account_numbers().await?;
//! # Ok(()) }
//! ```

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::auth::TokenManager;
use crate::error::{Error, Result};
use crate::models::account::{Account, AccountFields, AccountNumber, UserPreferences};
use crate::models::instruments::{Instrument, InstrumentsResponse, Projection};
use crate::models::market_hours::{Market, MarketHours};
use crate::models::movers::{Index, Mover, MoverFrequency, MoversResponse, SortOrder};
use crate::models::options::{OptionChain, OptionChainRequest};
use crate::models::orders::{GetOrdersRequest, Order, OrderId};
use crate::models::price_history::{PriceHistory, PriceHistoryRequest};
use crate::models::quotes::{QuoteFields, QuoteResponse, QuotesMap};
use crate::models::transactions::{GetTransactionsRequest, Transaction, TransactionId};

/// Base URL for trading / account endpoints.
const TRADER_BASE: &str = "https://api.schwabapi.com/trader/v1";
/// Base URL for market data endpoints.
const MARKETDATA_BASE: &str = "https://api.schwabapi.com/marketdata/v1";

/// Async REST client for the Schwab API.
///
/// Create once and clone the `Arc<TokenManager>` reference as needed.
pub struct SchwabClient {
    http: reqwest::Client,
    tokens: Arc<TokenManager>,
}

impl SchwabClient {
    /// Create a new client backed by the given token manager.
    pub fn new(tokens: Arc<TokenManager>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("schwab-api-rust/0.1")
                .build()
                .expect("failed to build reqwest client"),
            tokens,
        }
    }

    // ── private HTTP helpers ───────────────────────────────────────────────

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let token = self.tokens.get_valid_token().await?;
        let resp = self
            .http
            .get(url)
            .bearer_auth(&token)
            .send()
            .await?;
        Self::check_status_and_deserialize(resp).await
    }

    #[allow(dead_code)]
    async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        let token = self.tokens.get_valid_token().await?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await?;
        Self::check_status_and_deserialize(resp).await
    }

    #[allow(dead_code)]
    async fn post_empty<B: Serialize>(&self, url: &str, body: &B) -> Result<()> {
        let token = self.tokens.get_valid_token().await?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await?;
        Self::check_status(resp).await
    }

    async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<()> {
        let token = self.tokens.get_valid_token().await?;
        let resp = self
            .http
            .put(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await?;
        Self::check_status(resp).await
    }

    async fn delete(&self, url: &str) -> Result<()> {
        let token = self.tokens.get_valid_token().await?;
        let resp = self
            .http
            .delete(url)
            .bearer_auth(&token)
            .send()
            .await?;
        Self::check_status(resp).await
    }

    async fn check_status(resp: reqwest::Response) -> Result<()> {
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Api {
                status: status.as_u16(),
                body,
            })
        }
    }

    async fn check_status_and_deserialize<T: DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<T>().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Api {
                status: status.as_u16(),
                body,
            })
        }
    }

    // ── Accounts ───────────────────────────────────────────────────────────

    /// Return account number → hash mappings for all linked accounts.
    pub async fn get_account_numbers(&self) -> Result<Vec<AccountNumber>> {
        self.get(&format!("{TRADER_BASE}/accounts/accountNumbers")).await
    }

    /// Return details for a single account by its hash.
    pub async fn get_account(
        &self,
        account_hash: &str,
        fields: Option<AccountFields>,
    ) -> Result<Account> {
        let mut url = format!("{TRADER_BASE}/accounts/{account_hash}");
        if let Some(f) = fields {
            url.push_str(&format!("?fields={}", f.as_str()));
        }
        self.get(&url).await
    }

    /// Return details for all linked accounts.
    pub async fn get_accounts(&self, fields: Option<AccountFields>) -> Result<Vec<Account>> {
        let mut url = format!("{TRADER_BASE}/accounts");
        if let Some(f) = fields {
            url.push_str(&format!("?fields={}", f.as_str()));
        }
        self.get(&url).await
    }

    /// Return user preferences, including streaming connection credentials.
    pub async fn get_user_preferences(&self) -> Result<UserPreferences> {
        self.get(&format!("{TRADER_BASE}/userPreference")).await
    }

    // ── Quotes ────────────────────────────────────────────────────────────

    /// Return a single real-time quote.
    pub async fn get_quote(
        &self,
        symbol: &str,
        fields: Option<QuoteFields>,
    ) -> Result<QuoteResponse> {
        let encoded = url::form_urlencoded::byte_serialize(symbol.as_bytes()).collect::<String>();
        let mut url = format!("{MARKETDATA_BASE}/quotes/{encoded}");
        if let Some(f) = fields {
            url.push_str(&format!("?fields={}", f.as_str()));
        }
        // The endpoint returns a map keyed by symbol; extract the value.
        let map: QuotesMap = self.get(&url).await?;
        map.into_values()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                body: format!("empty response for symbol '{symbol}'"),
            })
    }

    /// Return real-time quotes for multiple symbols.
    pub async fn get_quotes(
        &self,
        symbols: &[&str],
        fields: Option<QuoteFields>,
        indicative: Option<bool>,
    ) -> Result<QuotesMap> {
        let symbols_param = symbols.join(",");
        let encoded =
            url::form_urlencoded::byte_serialize(symbols_param.as_bytes()).collect::<String>();
        let mut url = format!("{MARKETDATA_BASE}/quotes?symbols={encoded}");
        if let Some(f) = fields {
            url.push_str(&format!("&fields={}", f.as_str()));
        }
        if let Some(ind) = indicative {
            url.push_str(&format!("&indicative={ind}"));
        }
        self.get(&url).await
    }

    // ── Price History ─────────────────────────────────────────────────────

    /// Return historical OHLCV bars for a symbol.
    pub async fn get_price_history(&self, req: PriceHistoryRequest) -> Result<PriceHistory> {
        let mut url = format!(
            "{MARKETDATA_BASE}/pricehistory?symbol={}",
            url::form_urlencoded::byte_serialize(req.symbol.as_bytes()).collect::<String>()
        );
        if let Some(pt) = req.period_type {
            url.push_str(&format!("&periodType={}", pt.as_str()));
        }
        if let Some(p) = req.period {
            url.push_str(&format!("&period={p}"));
        }
        if let Some(ft) = req.frequency_type {
            url.push_str(&format!("&frequencyType={}", ft.as_str()));
        }
        if let Some(f) = req.frequency {
            url.push_str(&format!("&frequency={f}"));
        }
        if let Some(sd) = req.start_date {
            url.push_str(&format!("&startDate={sd}"));
        }
        if let Some(ed) = req.end_date {
            url.push_str(&format!("&endDate={ed}"));
        }
        if let Some(ext) = req.need_extended_hours_data {
            url.push_str(&format!("&needExtendedHoursData={ext}"));
        }
        if let Some(prev) = req.need_previous_close {
            url.push_str(&format!("&needPreviousClose={prev}"));
        }
        self.get(&url).await
    }

    /// Return daily bars for the past `period` trading days (default 10).
    pub async fn get_price_history_daily(
        &self,
        symbol: &str,
        period: Option<i32>,
    ) -> Result<PriceHistory> {
        use crate::models::price_history::{FrequencyType, PeriodType};
        self.get_price_history(PriceHistoryRequest {
            symbol: symbol.to_string(),
            period_type: Some(PeriodType::Day),
            period,
            frequency_type: Some(FrequencyType::Daily),
            frequency: Some(1),
            ..Default::default()
        })
        .await
    }

    /// Return 1-minute bars for the past `period` trading days (default 1).
    pub async fn get_price_history_every_minute(
        &self,
        symbol: &str,
        period: Option<i32>,
    ) -> Result<PriceHistory> {
        use crate::models::price_history::{FrequencyType, PeriodType};
        self.get_price_history(PriceHistoryRequest {
            symbol: symbol.to_string(),
            period_type: Some(PeriodType::Day),
            period,
            frequency_type: Some(FrequencyType::Minute),
            frequency: Some(1),
            ..Default::default()
        })
        .await
    }

    /// Return 5-minute bars for the past `period` trading days (default 1).
    pub async fn get_price_history_five_minutes(
        &self,
        symbol: &str,
        period: Option<i32>,
    ) -> Result<PriceHistory> {
        use crate::models::price_history::{FrequencyType, PeriodType};
        self.get_price_history(PriceHistoryRequest {
            symbol: symbol.to_string(),
            period_type: Some(PeriodType::Day),
            period,
            frequency_type: Some(FrequencyType::Minute),
            frequency: Some(5),
            ..Default::default()
        })
        .await
    }

    /// Return weekly bars for the past `period` years (default 1).
    pub async fn get_price_history_weekly(
        &self,
        symbol: &str,
        period: Option<i32>,
    ) -> Result<PriceHistory> {
        use crate::models::price_history::{FrequencyType, PeriodType};
        self.get_price_history(PriceHistoryRequest {
            symbol: symbol.to_string(),
            period_type: Some(PeriodType::Year),
            period,
            frequency_type: Some(FrequencyType::Weekly),
            frequency: Some(1),
            ..Default::default()
        })
        .await
    }

    // ── Options ───────────────────────────────────────────────────────────

    /// Return the full option chain for a symbol.
    pub async fn get_option_chain(&self, req: OptionChainRequest) -> Result<OptionChain> {
        let encoded =
            url::form_urlencoded::byte_serialize(req.symbol.as_bytes()).collect::<String>();
        let mut url = format!("{MARKETDATA_BASE}/chains?symbol={encoded}");
        if let Some(ct) = req.contract_type {
            url.push_str(&format!("&contractType={}", ct.as_str()));
        }
        if let Some(sc) = req.strike_count {
            url.push_str(&format!("&strikeCount={sc}"));
        }
        if let Some(iuq) = req.include_underlying_quote {
            url.push_str(&format!("&includeUnderlyingQuote={iuq}"));
        }
        if let Some(s) = req.strategy {
            url.push_str(&format!("&strategy={}", s.as_str()));
        }
        if let Some(i) = req.interval {
            url.push_str(&format!("&interval={i}"));
        }
        if let Some(st) = req.strike {
            url.push_str(&format!("&strike={st}"));
        }
        if let Some(r) = req.range {
            url.push_str(&format!("&range={}", r.as_str()));
        }
        if let Some(fd) = &req.from_date {
            url.push_str(&format!("&fromDate={fd}"));
        }
        if let Some(td) = &req.to_date {
            url.push_str(&format!("&toDate={td}"));
        }
        if let Some(v) = req.volatility {
            url.push_str(&format!("&volatility={v}"));
        }
        if let Some(up) = req.underlying_price {
            url.push_str(&format!("&underlyingPrice={up}"));
        }
        if let Some(ir) = req.interest_rate {
            url.push_str(&format!("&interestRate={ir}"));
        }
        if let Some(dte) = req.days_to_expiration {
            url.push_str(&format!("&daysToExpiration={dte}"));
        }
        if let Some(em) = req.exp_month {
            url.push_str(&format!("&expMonth={}", em.as_str()));
        }
        if let Some(ot) = req.option_type {
            url.push_str(&format!("&optionType={}", ot.as_str()));
        }
        if let Some(e) = &req.entitlement {
            url.push_str(&format!("&entitlement={e}"));
        }
        self.get(&url).await
    }

    // ── Instruments ───────────────────────────────────────────────────────

    /// Search for instruments by symbol or description.
    pub async fn get_instruments(
        &self,
        symbols: &[&str],
        projection: Projection,
    ) -> Result<Vec<Instrument>> {
        let symbol_param = symbols.join(",");
        let url = format!(
            "{MARKETDATA_BASE}/instruments?symbol={}&projection={}",
            url::form_urlencoded::byte_serialize(symbol_param.as_bytes()).collect::<String>(),
            projection.as_str()
        );
        let resp: InstrumentsResponse = self.get(&url).await?;
        Ok(resp.instruments.unwrap_or_default())
    }

    /// Return a single instrument by its CUSIP.
    pub async fn get_instrument_by_cusip(&self, cusip: &str) -> Result<Instrument> {
        let url = format!(
            "{MARKETDATA_BASE}/instruments/{}",
            url::form_urlencoded::byte_serialize(cusip.as_bytes()).collect::<String>()
        );
        self.get(&url).await
    }

    // ── Orders ────────────────────────────────────────────────────────────

    /// Place a new order for the specified account.
    ///
    /// Returns the assigned order ID extracted from the `Location` header.
    pub async fn place_order(&self, account_hash: &str, order: &Order) -> Result<OrderId> {
        let token = self.tokens.get_valid_token().await?;
        let url = format!("{TRADER_BASE}/accounts/{account_hash}/orders");
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(order)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                body,
            });
        }
        // The order ID is in the Location header: .../orders/<id>
        if let Some(loc) = resp.headers().get("location")
            && let Ok(s) = loc.to_str()
            && let Some(id_str) = s.rsplit('/').next()
            && let Ok(id) = id_str.parse::<OrderId>()
        {
            return Ok(id);
        }
        Err(Error::Api {
            status: status.as_u16(),
            body: "missing Location header in place_order response".to_string(),
        })
    }

    /// Fetch a single order by its ID.
    pub async fn get_order(&self, account_hash: &str, order_id: OrderId) -> Result<Order> {
        self.get(&format!(
            "{TRADER_BASE}/accounts/{account_hash}/orders/{order_id}"
        ))
        .await
    }

    /// Fetch all orders for one account matching the given filter.
    pub async fn get_orders_for_account(
        &self,
        account_hash: &str,
        req: GetOrdersRequest,
    ) -> Result<Vec<Order>> {
        let url = build_orders_url(
            &format!("{TRADER_BASE}/accounts/{account_hash}/orders"),
            &req,
        );
        self.get(&url).await
    }

    /// Fetch orders across all accounts matching the given filter.
    pub async fn get_orders_for_all_accounts(
        &self,
        req: GetOrdersRequest,
    ) -> Result<Vec<Order>> {
        let url = build_orders_url(&format!("{TRADER_BASE}/orders"), &req);
        self.get(&url).await
    }

    /// Cancel an order by ID.
    pub async fn cancel_order(&self, account_hash: &str, order_id: OrderId) -> Result<()> {
        self.delete(&format!(
            "{TRADER_BASE}/accounts/{account_hash}/orders/{order_id}"
        ))
        .await
    }

    /// Replace (modify) an existing order.
    pub async fn replace_order(
        &self,
        account_hash: &str,
        order_id: OrderId,
        order: &Order,
    ) -> Result<()> {
        self.put(
            &format!("{TRADER_BASE}/accounts/{account_hash}/orders/{order_id}"),
            order,
        )
        .await
    }

    // ── Transactions ──────────────────────────────────────────────────────

    /// Fetch a single transaction by its ID.
    pub async fn get_transaction(
        &self,
        account_hash: &str,
        tx_id: TransactionId,
    ) -> Result<Transaction> {
        self.get(&format!(
            "{TRADER_BASE}/accounts/{account_hash}/transactions/{tx_id}"
        ))
        .await
    }

    /// Fetch transactions for an account, optionally filtered.
    pub async fn get_transactions(
        &self,
        account_hash: &str,
        req: GetTransactionsRequest,
    ) -> Result<Vec<Transaction>> {
        let mut url = format!("{TRADER_BASE}/accounts/{account_hash}/transactions");
        let mut sep = '?';
        if let Some(tt) = req.transaction_type {
            url.push_str(&format!("{sep}types={}", tt.as_str()));
            sep = '&';
        }
        if let Some(sym) = &req.symbol {
            url.push_str(&format!("{sep}symbol={sym}"));
            sep = '&';
        }
        if let Some(sd) = req.start_date {
            url.push_str(&format!("{sep}startDate={}", format_dt(sd)));
            sep = '&';
        }
        if let Some(ed) = req.end_date {
            url.push_str(&format!("{sep}endDate={}", format_dt(ed)));
            let _ = sep; // suppress unused warning
        }
        self.get(&url).await
    }

    // ── Market Data ───────────────────────────────────────────────────────

    /// Return top movers for the specified index.
    pub async fn get_movers(
        &self,
        index: Index,
        sort: SortOrder,
        frequency: MoverFrequency,
    ) -> Result<Vec<Mover>> {
        let url = format!(
            "{MARKETDATA_BASE}/movers/{}?sort={}&frequency={}",
            url::form_urlencoded::byte_serialize(index.as_str().as_bytes())
                .collect::<String>(),
            sort.as_str(),
            frequency.as_u32(),
        );
        let resp: MoversResponse = self.get(&url).await?;
        Ok(resp.screeners.unwrap_or_default())
    }

    /// Return market hours for the specified markets on the given date.
    pub async fn get_market_hours(
        &self,
        markets: &[Market],
        date: chrono::NaiveDate,
    ) -> Result<MarketHours> {
        let markets_param = markets
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "{MARKETDATA_BASE}/markets?markets={}&date={}",
            markets_param,
            date.format("%Y-%m-%d")
        );
        self.get(&url).await
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn format_dt(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn build_orders_url(base: &str, req: &GetOrdersRequest) -> String {
    let mut url = base.to_string();
    let mut sep = '?';
    if let Some(from) = req.from_entered_time {
        url.push_str(&format!("{sep}fromEnteredTime={}", format_dt(from)));
        sep = '&';
    }
    if let Some(to) = req.to_entered_time {
        url.push_str(&format!("{sep}toEnteredTime={}", format_dt(to)));
        sep = '&';
    }
    if let Some(max) = req.max_results {
        url.push_str(&format!("{sep}maxResults={max}"));
        sep = '&';
    }
    if let Some(status) = req.status {
        // Serialize the status enum to its string form.
        let status_str = serde_json::to_string(&status)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        url.push_str(&format!("{sep}status={status_str}"));
        let _ = sep; // suppress unused warning
    }
    url
}
