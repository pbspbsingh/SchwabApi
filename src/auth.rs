//! OAuth 2.0 token management for the Schwab API.
//!
//! # Flow
//! 1. Build the authorization URL with [`TokenManager::authorize_url`].
//! 2. Direct the user to that URL; capture the `code` query parameter from the redirect.
//! 3. Call [`TokenManager::exchange_code`] to receive a [`TokenSet`].
//! 4. Construct a [`TokenManager`] via [`TokenManager::new`] and pass it to
//!    [`SchwabClient`][crate::SchwabClient] and [`StreamClient`][crate::StreamClient].
//! 5. Persist tokens with [`TokenManager::save`] and restore them with [`TokenManager::load`].

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{Error, Result};

const TOKEN_ENDPOINT: &str = "https://api.schwabapi.com/v1/oauth/token";
const AUTH_ENDPOINT: &str = "https://api.schwabapi.com/v1/oauth/authorize";

// ── OAuth configuration ────────────────────────────────────────────────────

/// Application credentials registered in the Schwab developer portal.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// The application key (client_id).
    pub app_key: String,
    /// The application secret (client_secret).
    pub app_secret: String,
    /// The redirect URI registered for this application.
    pub redirect_uri: String,
}

// ── Token set ─────────────────────────────────────────────────────────────

/// A set of OAuth tokens along with their expiry times.
///
/// Serialized to/from JSON for file-based persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    /// Bearer token for API requests (TTL ~30 min).
    pub access_token: String,
    /// Token used to obtain new access tokens (TTL ~7 days).
    pub refresh_token: String,
    /// UTC instant at which `access_token` expires.
    pub expires_at: DateTime<Utc>,
    /// UTC instant at which `refresh_token` expires.
    pub refresh_expires_at: DateTime<Utc>,
}

// ── Raw token response from the Schwab OAuth endpoint ─────────────────────

#[derive(Debug, Deserialize)]
struct RawTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,          // seconds until access token expiry
    refresh_token_expires_in: Option<i64>,
    token_type: String,
    // error fields (present on failure)
    error: Option<String>,
    error_description: Option<String>,
}

// ── Token manager ─────────────────────────────────────────────────────────

/// Manages OAuth token storage and automatic refresh.
///
/// Construct via [`TokenManager::new`] and share across the application using
/// `Arc<TokenManager>`.
pub struct TokenManager {
    config: OAuthConfig,
    tokens: RwLock<TokenSet>,
    http: reqwest::Client,
}

impl TokenManager {
    /// Create a new manager from a loaded/exchanged [`TokenSet`].
    pub fn new(config: OAuthConfig, tokens: TokenSet) -> Arc<Self> {
        Arc::new(Self {
            config,
            tokens: RwLock::new(tokens),
            http: reqwest::Client::new(),
        })
    }

    /// Build the URL to which the user should be directed to authorize the application.
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

    /// Exchange an authorization code for a [`TokenSet`].
    ///
    /// The code is the `code` query parameter from the OAuth redirect.
    pub async fn exchange_code(&self, code: &str) -> Result<TokenSet> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
        ];
        let token_set = self.post_token_request(&params).await?;
        *self.tokens.write().await = token_set.clone();
        Ok(token_set)
    }

    /// Return a valid access token, refreshing it first if it expires within 60 seconds.
    pub async fn get_valid_token(&self) -> Result<String> {
        // Fast path: read lock only.
        {
            let tokens = self.tokens.read().await;
            if tokens.expires_at - Utc::now() > Duration::seconds(60) {
                return Ok(tokens.access_token.clone());
            }
        }

        // Slow path: need to refresh — take write lock.
        let mut tokens = self.tokens.write().await;

        // Re-check after acquiring write lock (another task may have already refreshed).
        if tokens.expires_at - Utc::now() > Duration::seconds(60) {
            return Ok(tokens.access_token.clone());
        }

        // Check that the refresh token itself hasn't expired.
        if tokens.refresh_expires_at < Utc::now() {
            return Err(Error::TokenExpired);
        }

        tracing::info!("access token expiring soon, refreshing");
        let refresh_token = tokens.refresh_token.clone();
        let new_tokens = self
            .refresh_access_token_with_lock(&refresh_token)
            .await?;

        // Preserve the original refresh_token if the endpoint didn't return a new one.
        *tokens = new_tokens;
        Ok(tokens.access_token.clone())
    }

    /// Serialize the current [`TokenSet`] to a JSON file at `path`.
    pub async fn save(&self, path: &Path) -> Result<()> {
        let tokens = self.tokens.read().await;
        let json = serde_json::to_string_pretty(&*tokens)?;
        tokio::fs::write(path, json)
            .await
            .map_err(|e| Error::Api {
                status: 0,
                body: format!("failed to write token file: {e}"),
            })?;
        Ok(())
    }

    /// Deserialize a [`TokenSet`] from a JSON file at `path`.
    pub fn load(path: &Path) -> Result<TokenSet> {
        let data = std::fs::read(path).map_err(|e| Error::Api {
            status: 0,
            body: format!("failed to read token file: {e}"),
        })?;
        let token_set: TokenSet = serde_json::from_slice(&data)?;
        Ok(token_set)
    }

    // ── internal helpers ────────────────────────────────────────────────

    async fn refresh_access_token_with_lock(&self, refresh_token: &str) -> Result<TokenSet> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
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

        if let Some(err_code) = raw.error {
            return Err(Error::OAuth {
                code: err_code,
                message: raw.error_description.unwrap_or_default(),
            });
        }

        let _ = raw.token_type; // expected "Bearer"
        let now = Utc::now();
        let token_set = TokenSet {
            access_token: raw.access_token,
            refresh_token: raw.refresh_token.unwrap_or_default(),
            expires_at: now + Duration::seconds(raw.expires_in),
            refresh_expires_at: now
                + Duration::seconds(raw.refresh_token_expires_in.unwrap_or(604800)),
        };
        Ok(token_set)
    }
}
