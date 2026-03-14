use std::path::Path;
use std::sync::Arc;

use schwab_api::auth::{OAuthConfig, TokenManager};
use schwab_api::models::quotes::QuoteResponse;
use schwab_api::stream::fields::LevelOneEquityField;
use schwab_api::{SchwabClient, StreamClient};

const TOKEN_FILE: &str = "tokens.json";

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> schwab_api::Result<()> {
    tracing_subscriber::fmt::init();

    let config = OAuthConfig {
        app_key:       std::env::var("SCHWAB_APP_KEY").expect("SCHWAB_APP_KEY not set"),
        app_secret:    std::env::var("SCHWAB_APP_SECRET").expect("SCHWAB_APP_SECRET not set"),
        redirect_uri:  std::env::var("SCHWAB_REDIRECT_URI").unwrap_or("https://127.0.0.1:8443".to_string()),
        tls_cert_path: std::env::var("SCHWAB_TLS_CERT").unwrap_or("cert.pem".to_string()).into(),
        tls_key_path:  std::env::var("SCHWAB_TLS_KEY").unwrap_or("key.pem".to_string()).into(),
    };

    // ── Auth ──────────────────────────────────────────────────────────────────
    let tokens = TokenManager::create(config, Path::new(TOKEN_FILE)).await?;

    // ── REST: fetch a snapshot AAPL quote ────────────────────────────────────
    let client = SchwabClient::new(Arc::clone(&tokens));

    let quotes = client.get_quotes(&["AAPL"], None, None).await?;
    if let Some(QuoteResponse::Equity(q)) = quotes.get("AAPL") {
        let quote = q.quote.as_ref();
        println!(
            "AAPL snapshot  bid={:.2}  ask={:.2}  last={:.2}  volume={}",
            quote.and_then(|q| q.bid_price).unwrap_or(0.0),
            quote.and_then(|q| q.ask_price).unwrap_or(0.0),
            quote.and_then(|q| q.last_price).unwrap_or(0.0),
            quote.and_then(|q| q.total_volume).unwrap_or(0),
        );
    }

    // ── Streaming: subscribe to AAPL L1 prices ───────────────────────────────
    let prefs = client.get_user_preferences().await?;
    let stream = StreamClient::connect(Arc::clone(&tokens), prefs).await?;

    let fields = [
        LevelOneEquityField::BidPrice,
        LevelOneEquityField::AskPrice,
        LevelOneEquityField::LastPrice,
        LevelOneEquityField::TotalVolume,
        LevelOneEquityField::NetChange,
    ];

    let mut rx = stream
        .level_one_equities()
        .subscribe(&["AAPL"], &fields, 128)
        .await?;

    println!("\nStreaming AAPL L1 (Ctrl+C to stop)...\n");

    while let Some(event) = rx.recv().await {
        println!(
            "[{}]  bid={:>8}  ask={:>8}  last={:>8}  vol={:>12}  chg={:>+.2}",
            event.symbol,
            event.bid_price.map_or("–".to_string(), |v| format!("{v:.2}")),
            event.ask_price.map_or("–".to_string(), |v| format!("{v:.2}")),
            event.last_price.map_or("–".to_string(), |v| format!("{v:.2}")),
            event.total_volume.map_or("–".to_string(), |v| v.to_string()),
            event.net_change.unwrap_or(0.0),
        );
    }

    Ok(())
}
