use std::env;

use rust_decimal::Decimal;
use schwab_api::orders::equity_buy_limit;
use schwab_api::{Error, Result, Symbol};

use super::support;

#[tokio::test]
#[ignore = "requires SCHWAB_LIVE_TESTS=1 and SCHWAB_LIVE_PREVIEW=1"]
async fn preview_typed_order_without_submission() -> Result<()> {
    if env::var("SCHWAB_LIVE_PREVIEW").as_deref() != Ok("1") {
        return Err(Error::Api {
            status: 0,
            body: "set SCHWAB_LIVE_PREVIEW=1 to enable order preview".into(),
        });
    }

    let client = support::client().await?;
    let account = client.get_account_numbers().await?.into_iter().next().ok_or_else(|| Error::Api {
        status: 0,
        body: "the token has no linked accounts".into(),
    })?;
    let symbol = env::var("SCHWAB_LIVE_SYMBOL").unwrap_or_else(|_| "AAPL".into());
    let order = equity_buy_limit(Symbol(symbol), Decimal::ONE, Decimal::new(1, 2))?;

    client.preview_order(&account.hash_value, &order).await?;
    Ok(())
}
