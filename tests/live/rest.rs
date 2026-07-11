use std::env;

use chrono::Utc;
use schwab_api::models::{
    Index, Market, MoverFrequency, OptionChainRequest, Projection, SortOrder,
};
use schwab_api::Result;

use super::support;

#[tokio::test]
#[ignore = "requires an existing Schwab OAuth token and SCHWAB_LIVE_TESTS=1"]
async fn read_only_endpoints_deserialize_live_responses() -> Result<()> {
    let client = support::client().await?;
    let symbol = env::var("SCHWAB_LIVE_SYMBOL").unwrap_or_else(|_| "AAPL".to_string());

    let accounts = client.get_account_numbers().await?;
    assert!(!accounts.is_empty(), "the token has no linked accounts");
    let account_hash = &accounts[0].hash_value;

    client.get_account(account_hash, None).await?;
    client.get_accounts(None).await?;
    client.get_user_preferences().await?;
    client.get_quote(&symbol, None).await?;
    client.get_quotes(&[&symbol], None, None).await?;
    client.get_price_history_daily(&symbol, Some(10)).await?;
    client.get_price_history_ten_minutes(&symbol, Some(1)).await?;
    client.get_option_chain(OptionChainRequest {
        symbol: symbol.clone(),
        ..Default::default()
    }).await?;
    client.get_option_expiration_chain(&symbol).await?;
    client.get_instruments(&[&symbol], Projection::SymbolSearch).await?;
    client.get_movers(Index::Sp500, SortOrder::Volume, MoverFrequency::Zero).await?;
    client.get_market_hours(&[Market::Equity], Utc::now().date_naive()).await?;

    Ok(())
}
