use std::env;
use std::time::Duration;

use schwab_api::stream::fields::LevelOneEquityField;
use schwab_api::{Result, SchwabClient, StreamClient};

use super::support;

#[tokio::test]
#[ignore = "requires an existing Schwab OAuth token and SCHWAB_LIVE_TESTS=1"]
async fn streaming_login_and_subscription_acknowledge() -> Result<()> {
    let tokens = support::tokens().await?;
    let client = SchwabClient::new(tokens.clone());
    let preferences = client.get_user_preferences().await?;
    let stream = StreamClient::connect(tokens, preferences).await?;
    let symbol = env::var("SCHWAB_LIVE_SYMBOL").unwrap_or_else(|_| "AAPL".to_string());

    tokio::time::timeout(
        Duration::from_secs(20),
        stream.level_one_equities().subscribe(&[&symbol], &[LevelOneEquityField::BidPrice], 8),
    )
    .await
    .map_err(|_| schwab_api::Error::StreamDisconnected)??;

    stream.logout().await
}
