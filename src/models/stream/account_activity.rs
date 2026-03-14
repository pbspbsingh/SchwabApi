//! Account activity streaming event structs.

use serde_json::Value;

use crate::error::{Error, Result};

/// A streaming account activity event from the ACCT_ACTIVITY service.
///
/// Schwab delivers account activity (fills, cancellations, etc.) as XML
/// embedded in field 3. The raw XML string is exposed here.
#[derive(Debug, Clone, Default)]
pub struct AccountActivityEvent {
    /// Field 0 — subscription key (always present).
    pub subscription_key: String,
    /// Field 1 — account number.
    pub account_number: Option<String>,
    /// Field 2 — activity message type (e.g. "OrderFill", "OrderCancel").
    pub message_type: Option<String>,
    /// Field 3 — raw XML activity payload.
    pub message_data: Option<String>,
}

impl TryFrom<&Value> for AccountActivityEvent {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::Api { status: 0, body: "expected a JSON object".to_string() })?;
        let mut e = AccountActivityEvent::default();
        for (k, val) in obj {
            match k.as_str() {
                "0" => e.subscription_key = val.as_str().unwrap_or_default().to_string(),
                "1" => e.account_number = val.as_str().map(|s| s.to_string()),
                "2" => e.message_type = val.as_str().map(|s| s.to_string()),
                "3" => e.message_data = val.as_str().map(|s| s.to_string()),
                _   => {}
            }
        }
        Ok(e)
    }
}
