//! Internal WebSocket wire-format types.
//!
//! These types are NOT part of the public API; they exist solely to
//! serialize outbound commands and deserialize inbound frames.

use serde::{Deserialize, Serialize};

// ── Outbound ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(super) struct WireRequest<'a> {
    pub requests: Vec<WireRequestItem<'a>>,
}

#[derive(Serialize)]
pub(super) struct WireRequestItem<'a> {
    pub service: &'a str,
    pub requestid: String,
    pub command: &'a str,
    #[serde(rename = "SchwabClientCustomerId")]
    pub customer_id: &'a str,
    #[serde(rename = "SchwabClientCorrelId")]
    pub correl_id: &'a str,
    pub parameters: serde_json::Value,
}

// ── Inbound ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(super) struct WireIncoming {
    pub response: Option<Vec<WireResponse>>,
    pub data: Option<Vec<WireData>>,
    pub notify: Option<Vec<WireNotify>>,
}

#[derive(Deserialize)]
pub(super) struct WireResponse {
    #[allow(dead_code)]
    pub requestid: String,
    #[allow(dead_code)]
    pub service: String,
    #[allow(dead_code)]
    pub command: String,
    pub content: WireResponseContent,
}

#[derive(Deserialize)]
pub(super) struct WireResponseContent {
    pub code: i32,
    pub msg: String,
}

#[derive(Deserialize)]
pub(super) struct WireData {
    pub service: String,
    #[allow(dead_code)]
    pub command: String,
    pub content: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub(super) struct WireNotify {
    pub heartbeat: Option<i64>,
}
