//! OAuth 2.0 token management for the Schwab API.
//!
//! # Usage
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
//! // First run: prints the authorize URL and prompts for the code.
//! // Subsequent runs: loads tokens from file silently.
//! let tokens = TokenManager::create(config, Path::new("tokens.json")).await?;
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
    /// The redirect URI registered for this application (e.g. `https://127.0.0.1:8443`).
    pub redirect_uri: String,
    /// Path to the TLS certificate file (PEM) used by the local callback server.
    pub tls_cert_path: PathBuf,
    /// Path to the TLS private key file (PEM) used by the local callback server.
    pub tls_key_path: PathBuf,
}

// ── TokenSet (internal) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenSet {
    access_token:       String,
    refresh_token:      String,
    expires_at:         DateTime<Utc>,
    refresh_expires_at: DateTime<Utc>,
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

/// Opaque token manager — pass to [`SchwabClient`][crate::SchwabClient] and
/// [`StreamClient`][crate::StreamClient]. Tokens refresh automatically.
pub struct TokenManager {
    config:     OAuthConfig,
    token_path: PathBuf,
    tokens:     RwLock<Option<TokenSet>>,
    http:       reqwest::Client,
}

impl TokenManager {
    /// Return a ready-to-use `Arc<TokenManager>`.
    ///
    /// - If `token_path` exists and the refresh token is still valid, tokens
    ///   are loaded silently.
    /// - Otherwise the OAuth flow is started: the authorize URL is printed to
    ///   stdout, the user is prompted to paste the redirect code, and the
    ///   resulting tokens are saved to `token_path` automatically.
    pub async fn create(config: OAuthConfig, token_path: &Path) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            token_path: token_path.to_path_buf(),
            tokens: RwLock::new(None),
            http: reqwest::Client::new(),
            config,
        });

        // Try loading an existing token file.
        if token_path.exists() {
            match manager.try_load().await {
                Ok(()) => {
                    tracing::info!("loaded tokens from '{}'", token_path.display());
                    return Ok(manager);
                }
                Err(e) => {
                    tracing::warn!("token file invalid or expired ({e}), re-authorizing");
                }
            }
        }

        // No valid tokens — run the interactive OAuth flow.
        manager.run_oauth_flow().await?;
        Ok(manager)
    }

    // ── pub(crate) ────────────────────────────────────────────────────────────

    /// Return a valid access token, refreshing silently if within 60 s of expiry.
    pub(crate) async fn get_valid_token(&self) -> Result<String> {
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

        // Re-check: another task may have already refreshed.
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

        // Persist in the background — don't block the caller.
        let serialized = serde_json::to_string_pretty(&*tokens)?;
        let path = self.token_path.clone();
        tokio::spawn(async move {
            if let Err(e) = tokio::fs::write(&path, serialized).await {
                tracing::warn!("failed to save refreshed tokens to '{}': {e}", path.display());
            }
        });

        Ok(access)
    }

    // ── private ───────────────────────────────────────────────────────────────

    async fn try_load(&self) -> Result<()> {
        let data = tokio::fs::read(&self.token_path).await.map_err(|e| Error::Api {
            status: 0,
            body: format!("read error: {e}"),
        })?;
        let token_set: TokenSet = serde_json::from_slice(&data)?;
        if token_set.refresh_expires_at < Utc::now() {
            return Err(Error::TokenExpired);
        }
        *self.tokens.write().await = Some(token_set);
        Ok(())
    }

    async fn run_oauth_flow(&self) -> Result<()> {
        use std::sync::Arc;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;
        use tokio_rustls::TlsAcceptor;

        // Ensure the redirect URI ends with '/' — Schwab won't follow the
        // redirect otherwise.
        let redirect_uri = if self.config.redirect_uri.ends_with('/') {
            self.config.redirect_uri.clone()
        } else {
            format!("{}/", self.config.redirect_uri)
        };

        // Parse host and port from the redirect URI.
        let redirect = url::Url::parse(&redirect_uri).map_err(|e| Error::Api {
            status: 0,
            body: format!("invalid redirect_uri: {e}"),
        })?;
        let host = redirect.host_str().unwrap_or("127.0.0.1").to_string();
        let port = redirect.port().unwrap_or(8443);
        let bind_addr = format!("{host}:{port}");

        // Load TLS cert and key from the paths provided in config.
        let cert_pem = tokio::fs::read(&self.config.tls_cert_path).await?;
        let key_pem  = tokio::fs::read(&self.config.tls_key_path).await?;

        let certs = rustls_pemfile::certs(&mut cert_pem.as_slice())
            .collect::<std::io::Result<Vec<_>>>()?;
        let key = rustls_pemfile::private_key(&mut key_pem.as_slice())?
            .ok_or_else(|| Error::Api {
                status: 0,
                body: format!("no private key found in '{}'", self.config.tls_key_path.display()),
            })?;

        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Api { status: 0, body: format!("TLS config failed: {e}") })?;
        let acceptor = TlsAcceptor::from(Arc::new(tls_config));

        // Bind before printing the URL so we're ready to receive the redirect.
        let listener = TcpListener::bind(&bind_addr).await.map_err(|e| Error::Api {
            status: 0,
            body: format!("failed to bind {bind_addr}: {e}"),
        })?;

        // Print the authorize URL for the user to open manually.
        let auth_url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}",
            AUTH_ENDPOINT,
            url::form_urlencoded::byte_serialize(self.config.app_key.as_bytes())
                .collect::<String>(),
            url::form_urlencoded::byte_serialize(redirect_uri.as_bytes())
                .collect::<String>(),
        );
        println!("\nOpen this URL in your browser to authorize:\n\n  {auth_url}\n");

        // Accept connections in a loop — the browser may fail the TLS handshake
        // on the first attempt (e.g. untrusted cert) and retry after the user
        // accepts the certificate warning.
        let code = loop {
            let (tcp, _) = listener.accept().await?;
            let mut tls = match acceptor.accept(tcp).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("TLS handshake failed (browser may need to accept the certificate): {e}");
                    continue;
                }
            };

            // Read the HTTP request and extract the `code` query parameter.
            let mut buf = vec![0u8; 4096];
            let n = tls.read(&mut buf).await?;
            let request = String::from_utf8_lossy(&buf[..n]);

            // First request line: "GET /?code=ABC123&session=XYZ HTTP/1.1"
            let Some(code) = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|path| path.split_once('?').map(|(_, q)| q))
                .and_then(|query| {
                    url::form_urlencoded::parse(query.as_bytes())
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.into_owned())
                })
            else {
                // Could be a browser preflight or favicon request — keep waiting.
                tracing::debug!("ignoring request without `code` parameter");
                continue;
            };

            // Respond so the browser shows a success page.
            let body = "<html><body><h1>Authorization successful — you can close this tab.</h1></body></html>";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            let _ = tls.write_all(response.as_bytes()).await;
            break code;
        };

        // Exchange the code for tokens.
        let params = [
            ("grant_type",   "authorization_code"),
            ("code",         code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ];
        let token_set = self.post_token_request(&params).await?;
        self.store_and_save(token_set).await
    }

    async fn store_and_save(&self, token_set: TokenSet) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&token_set)?;
        tokio::fs::write(&self.token_path, &serialized)
            .await
            .map_err(|e| Error::Api {
                status: 0,
                body: format!("failed to write '{}': {e}", self.token_path.display()),
            })?;
        *self.tokens.write().await = Some(token_set);
        Ok(())
    }

    async fn do_refresh(&self, refresh_token: &str) -> Result<TokenSet> {
        let params = [
            ("grant_type",    "refresh_token"),
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
            refresh_expires_at: now + Duration::seconds(
                raw.refresh_token_expires_in.unwrap_or(604_800),
            ),
        })
    }
}
