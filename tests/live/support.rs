use std::env;
use std::path::Path;
use std::sync::Arc;

use schwab_api::auth::{OAuthConfig, TokenManager};
use schwab_api::{Error, Result, SchwabClient};

pub fn required(name: &str) -> Result<String> {
    env::var(name).map_err(|_| Error::Api {
        status: 0,
        body: format!("{name} must be set for live tests"),
    })
}

pub async fn tokens() -> Result<Arc<TokenManager>> {
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
    TokenManager::create(config, Path::new(&token_path)).await
}

pub async fn client() -> Result<SchwabClient> {
    Ok(SchwabClient::new(tokens().await?))
}
