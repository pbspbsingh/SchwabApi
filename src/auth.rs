//! OAuth 2.0 token management for the Schwab API.
//!
//! # First-time setup
//! ```no_run
//! # async fn example() -> schwab_api::Result<()> {
//! use schwab_api::auth::{OAuthConfig, TokenManager};
//! use std::path::Path;
//!
//! let config = OAuthConfig {
//!     app_key:      "your_app_key".to_string(),
//!     app_secret:   "your_app_secret".to_string(),
//!     redirect_uri: "https://127.0.0.1".to_string(),
//! };
//!
//! let manager = TokenManager::new(config, Path::new("tokens.json"));
//! println!("Visit: {}", manager.authorize_url());
//! // paste the `code` query parameter from the redirect URL:
//! manager.exchange_code("paste-code-here").await?;
//! // tokens.json is now saved automatically
//! # Ok(()) }
//! ```
//!
//! # Subsequent runs
//! ```no_run
//! # async fn example() -> schwab_api::Result<()> {
//! use schwab_api::auth::{OAuthConfig, TokenManager};
//! use std::path::Path;
//!
//! let config = OAuthConfig { /* ... */ # app_key: String::new(), app_secret: String::new(), redirect_uri: String::new() };
//! let manager = TokenManager::load(config, Path::new("tokens.json"))?;
//! // tokens are loaded and will refresh automatically
//! # Ok(()) }
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{Error, Result};

const TOKEN_ENDPOINT: &str = "https://api.schwabapi.com/v1/oauth/token";
const AUTH_ENDPOINT:  &str = "https://api.schwabapi.com/v1/oauth/authorize";

// ── OAuth configuration ───────────────────────────────────────────────────────

/// Application credentials registered in the Schwab developer portal.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// The application key (client_id).
    pub app_key: String,
    /// The application secret (client_secret).
    pub app_secret: String,
    /// The redirect URI registered for this application (must be `https://127.0.0.1`).
    pub redirect_uri: String,
}

// ── TokenSet (internal) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenSet {
    access_token:        String,
    refresh_token:       String,
    expires_at:          DateTime<Utc>,
    refresh_expires_at:  DateTime<Utc>,
}

// ── Raw token response from Schwab OAuth endpoint ────────────────────────────

#[derive(Debug, Deserialize)]
struct RawTokenResponse {
    access_token:             Option<String>,
    refresh_token:            Option<String>,
    expires_in:               Option<i64>,
    refresh_token_expires_in: Option<i64>,
    error:                    Option<String>,
    error_description:        Option<String>,
}

// ── TokenManager ─────────────────────────────────────────────────────────────

/// Manages OAuth tokens for the Schwab API.
///
/// Tokens are persisted automatically to the file provided at construction.
/// Access tokens are refreshed transparently before expiry; no manual token
/// handling is required.
///
/// Share across the application via `Arc<TokenManager>`.
pub struct TokenManager {
    config:     OAuthConfig,
    token_path: PathBuf,
    tokens:     RwLock<Option<TokenSet>>,
    http:       reqwest::Client,
}

impl TokenManager {
    /// Create a new manager for **first-time setup**.
    ///
    /// No token file is required to exist yet. Call [`authorize_url`][Self::authorize_url]
    /// to get the login URL, then [`exchange_code`][Self::exchange_code] to complete
    /// authorization. Tokens are saved to `token_path` automatically.
    pub fn new(config: OAuthConfig, token_path: &Path) -> Arc<Self> {
        Arc::new(Self {
            config,
            token_path: token_path.to_path_buf(),
            tokens: RwLock::new(None),
            http: reqwest::Client::new(),
        })
    }

    /// Load an existing token file and create a manager ready for immediate use.
    ///
    /// Returns an error if the file does not exist or cannot be parsed.
    /// If the refresh token has expired, an error is returned and the user
    /// must re-authorize via [`new`][Self::new] + [`exchange_code`][Self::exchange_code].
    pub fn load(config: OAuthConfig, token_path: &Path) -> Result<Arc<Self>> {
        let data = std::fs::read(token_path).map_err(|e| Error::Api {
            status: 0,
            body: format!("failed to read token file '{}': {e}", token_path.display()),
        })?;
        let tokens: TokenSet = serde_json::from_slice(&data)?;

        if tokens.refresh_expires_at < Utc::now() {
            return Err(Error::TokenExpired);
        }

        Ok(Arc::new(Self {
            config,
            token_path: token_path.to_path_buf(),
            tokens: RwLock::new(Some(tokens)),
            http: reqwest::Client::new(),
        }))
    }

    /// Build the URL to which the user must navigate to authorize the application.
    pub fn authorize_url(&self) -> String {
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}",
            AUTH_ENDPOINT,
            url::form_urlencoded::byte_serialize(self.config.app_key.as_bytes())
                .collect::<String>(),
            url::form_urlencoded::byte_serialize(self.config.redirect_uri.as_bytes())
                .collect::<String>(),
        )
    }

    /// Exchange the authorization code from the OAuth redirect for tokens.
    ///
    /// The code is the `code` query parameter in the redirect URL.
    /// Tokens are saved to the token file automatically.
    pub async fn exchange_code(&self, code: &str) -> Result<()> {
        let params = [
            ("grant_type",    "authorization_code"),
            ("code",          code),
            ("redirect_uri",  &self.config.redirect_uri),
        ];
        let token_set = self.post_token_request(&params).await?;
        self.store_and_save(token_set).await
    }

    /// Return a valid access token string, refreshing silently if needed.
    ///
    /// Called internally by [`SchwabClient`][crate::SchwabClient] and
    /// [`StreamClient`][crate::StreamClient]; application code rarely needs
    /// to call this directly.
    pub async fn get_valid_token(&self) -> Result<String> {
        // Fast path: read lock only.
        {
            let guard = self.tokens.read().await;
            let tokens = guard.as_ref().ok_or(Error::TokenExpired)?;
            if tokens.expires_at - Utc::now() > Duration::seconds(60) {
                return Ok(tokens.access_token.clone());
            }
        }

        // Slow path: token is expiring — take write lock.
        let mut guard = self.tokens.write().await;
        let tokens = guard.as_mut().ok_or(Error::TokenExpired)?;

        // Re-check: another task may have refreshed while we waited.
        if tokens.expires_at - Utc::now() > Duration::seconds(60) {
            return Ok(tokens.access_token.clone());
        }

        if tokens.refresh_expires_at < Utc::now() {
            return Err(Error::TokenExpired);
        }

        tracing::info!("access token expiring soon, refreshing");
        let refresh_token = tokens.refresh_token.clone();
        let new_tokens = self.do_refresh(&refresh_token).await?;
        let access = new_tokens.access_token.clone();
        *tokens = new_tokens;

        // Persist updated tokens in the background — don't block the caller.
        let serialized = serde_json::to_string_pretty(&*tokens)?;
        let path = self.token_path.clone();
        tokio::spawn(async move {
            if let Err(e) = tokio::fs::write(&path, serialized).await {
                tracing::warn!("failed to persist refreshed tokens to '{}': {e}", path.display());
            }
        });

        Ok(access)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    async fn store_and_save(&self, token_set: TokenSet) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&token_set)?;
        tokio::fs::write(&self.token_path, serialized)
            .await
            .map_err(|e| Error::Api {
                status: 0,
                body: format!(
                    "failed to write token file '{}': {e}",
                    self.token_path.display()
                ),
            })?;
        *self.tokens.write().await = Some(token_set);
        Ok(())
    }

    async fn do_refresh(&self, refresh_token: &str) -> Result<TokenSet> {
        let params = [
            ("grant_type",     "refresh_token"),
            ("refresh_token",  refresh_token),
        ];
        self.post_token_request(&params).await
    }

    async fn post_token_request(&self, params: &[(&str, &str)]) -> Result<TokenSet> {
        let resp = self
            .http
            .post(TOKEN_ENDPOINT)
            .basic_auth(&self.config.app_key, Some(&self.config.app_secret))
            .form(params)
            .send()
            .await?;

        let raw: RawTokenResponse = resp.json().await?;

        if let Some(code) = raw.error {
            return Err(Error::OAuth {
                code,
                message: raw.error_description.unwrap_or_default(),
            });
        }

        let now = Utc::now();
        Ok(TokenSet {
            access_token:       raw.access_token.unwrap_or_default(),
            refresh_token:      raw.refresh_token.unwrap_or_default(),
            expires_at:         now + Duration::seconds(raw.expires_in.unwrap_or(1800)),
            refresh_expires_at: now + Duration::seconds(raw.refresh_token_expires_in.unwrap_or(604_800)),
        })
    }
}
