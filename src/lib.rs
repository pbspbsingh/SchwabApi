pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod orders;
pub mod stream;
pub mod types;

pub use auth::{OAuthConfig, TokenManager};
pub use client::SchwabClient;
pub use error::{Error, Result};
pub use stream::StreamClient;
pub use types::{AccountHash, Cusip, Money, Symbol, Timestamp};

// ── Shared HTTP client ────────────────────────────────────────────────────────

static HTTP_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

/// Returns the crate-wide shared `reqwest::Client` (initialised once).
fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("schwab-api-rust/0.1")
            .build()
            .expect("failed to build HTTP client")
    })
}
