use std::env;
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use schwab_api::auth::{OAuthConfig, TokenManager};
use schwab_api::models::{
    Index, Market, MoverFrequency, OptionChainRequest, Projection, SortOrder,
};
use schwab_api::{Error, Result, SchwabClient};

fn required(name: &str) -> Result<String> {
    env::var(name).map_err(|_| Error::Api {
        status: 0,
        body: format!("{name} must be set for live tests"),
    })
}

async fn live_client() -> Result<SchwabClient> {
    if env::var("SCHWAB_LIVE_TESTS").as_deref() != Ok("1") {
        return Err(Error::Api {
            status: 0,
            body: "set SCHWAB_LIVE_TESTS=1 to run live tests".to_string(),
        });
    }

    let config = OAuthConfig {
        app_key: required("SCHWAB_APP_KEY")?,
        app_secret: required("SCHWAB_APP_SECRET")?,
        redirect_uri: required("SCHWAB_REDIRECT_URI")?,
        tls_cert_path: required("SCHWAB_TLS_CERT")?.into(),
        tls_key_path: required("SCHWAB_TLS_KEY")?.into(),
    };
    let token_path = required("SCHWAB_TOKEN_PATH")?;
    let tokens = TokenManager::create(config, Path::new(&token_path)).await?;
    Ok(SchwabClient::new(Arc::clone(&tokens)))
}

#[tokio::test]
#[ignore = "requires an existing Schwab OAuth token and SCHWAB_LIVE_TESTS=1"]
async fn read_only_rest_endpoints() -> Result<()> {
    let client = live_client().await?;
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
