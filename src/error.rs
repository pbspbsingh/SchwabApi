/// Unified error type for all schwab_api operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An underlying HTTP transport error from reqwest.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A WebSocket protocol error from tungstenite.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// An I/O error (e.g. from the local TLS callback server).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// OAuth flow error returned by the Schwab authorization server.
    #[error("OAuth error: {code} \u{2014} {message}")]
    OAuth { code: String, message: String },

    /// The streaming LOGIN command was rejected by the server.
    #[error("Stream login failed: code={code}, msg={msg}")]
    StreamLoginFailed { code: i32, msg: String },

    /// A response frame arrived for an unexpected request ID.
    #[error("Unexpected stream response for requestid={requestid}")]
    UnexpectedStreamResponse { requestid: String },

    /// A SUBS/ADD command was rejected by the server.
    #[error("Stream subscription failed: code={code}, msg={msg}")]
    SubscriptionFailed { code: i32, msg: String },

    /// Caller attempted a second `subscribe()` on the same service.
    #[error("Already subscribed to service '{service}' \u{2014} use add_symbols() to expand")]
    AlreadySubscribed { service: &'static str },

    /// Caller attempted `add_symbols()` or `unsubscribe()` without a prior `subscribe()`.
    #[error("Not subscribed to service '{service}' \u{2014} call subscribe() first")]
    NotSubscribed { service: &'static str },

    /// The WebSocket connection was closed unexpectedly.
    #[error("Stream disconnected")]
    StreamDisconnected,

    /// The refresh token has expired; full re-authentication is required.
    #[error("Token expired \u{2014} re-authentication required")]
    TokenExpired,

    /// The Schwab REST API returned a non-2xx HTTP status code.
    #[error("API error: status={status}, body={body}")]
    Api { status: u16, body: String },
}

/// Convenience alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
