//! Shared domain primitives and wire-format adapters.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Exact decimal value used for prices, balances, quantities, and ratios.
pub type Money = Decimal;

/// A Schwab instrument symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Symbol(pub String);

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str { &self.0 }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

/// The opaque identifier required by account-specific endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountHash(pub String);

impl AsRef<str> for AccountHash {
    fn as_ref(&self) -> &str { &self.0 }
}

/// UTC timestamp encoded by Schwab as Unix milliseconds or an RFC 3339 string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub DateTime<Utc>);

impl Deref for Timestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self { Self(value) }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_i64(self.0.timestamp_millis())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireTimestamp { Millis(i64), Text(String) }

        match WireTimestamp::deserialize(deserializer)? {
            WireTimestamp::Millis(value) => Utc.timestamp_millis_opt(value)
                .single()
                .map(Timestamp)
                .ok_or_else(|| serde::de::Error::custom("invalid Unix millisecond timestamp")),
            WireTimestamp::Text(value) => DateTime::parse_from_rfc3339(&value)
                .map(|time| Timestamp(time.with_timezone(&Utc)))
                .or_else(|_| DateTime::from_str(&value).map(Timestamp))
                .map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::Timestamp;

    #[test]
    fn timestamp_deserializes_milliseconds_and_rfc3339() {
        let millis: Timestamp = serde_json::from_str("1718308800000").unwrap();
        assert_eq!(millis.0, Utc.timestamp_millis_opt(1_718_308_800_000).single().unwrap());

        let text: Timestamp = serde_json::from_str("\"2024-06-13T12:00:00Z\"").unwrap();
        assert_eq!(text.0, Utc.with_ymd_and_hms(2024, 6, 13, 12, 0, 0).unwrap());
    }
}
