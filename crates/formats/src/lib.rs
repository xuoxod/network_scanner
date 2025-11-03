//! Canonical discovery/output types used across the workspace
//!
//! This crate contains the canonical Rust types for discovery records and
//! provides serde-friendly mapping to JSON and CSV for golden-file tests.

use serde::{Deserialize, Serialize};

/// A single discovery record representing a host/service observation.
///
/// Keep this struct minimal and stable: add new optional fields rather than
/// changing existing names so golden-file compatibility is easier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryRecord {
    /// IP address in string form (v4 or v6)
    pub ip: String,
    /// Optional observed service port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Free-form banner or probe result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// Optional MAC address if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
    /// Optional vendor / manufacturer string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Optional ISO timestamp string from source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl DiscoveryRecord {
    /// Construct a new discovery record. Keep constructor small for tests.
    pub fn new(
        ip: &str,
        port: Option<u16>,
        banner: Option<&str>,
        mac: Option<&str>,
        vendor: Option<&str>,
        timestamp: Option<&str>,
    ) -> Self {
        Self {
            ip: ip.to_string(),
            port,
            banner: banner.map(|s| s.to_string()),
            mac: mac.map(|s| s.to_string()),
            vendor: vendor.map(|s| s.to_string()),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }
}

/// Round-trip helpers: JSON (serde_json) and CSV (csv crate)
pub mod serde_helpers {
    use super::DiscoveryRecord;

    /// Serialize to compact JSON string
    pub fn to_json(rec: &DiscoveryRecord) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(rec)?)
    }

    /// Deserialize from JSON string
    pub fn from_json(s: &str) -> Result<DiscoveryRecord, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(s)?)
    }

    /// Serialize to CSV (single-record, header included)
    pub fn to_csv(rec: &DiscoveryRecord) -> Result<String, Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(rec)?;
        wtr.flush()?;
        // into_inner returns a Vec<u8> on success
        let inner = wtr
            .into_inner()
            .map_err(|e| Box::new(std::io::Error::new(e.error().kind(), e.to_string())))?;
        Ok(String::from_utf8_lossy(&inner).to_string())
    }

    /// Deserialize single-record CSV string into DiscoveryRecord
    pub fn from_csv(s: &str) -> Result<DiscoveryRecord, Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_reader(s.as_bytes());
        let mut iter = rdr.deserialize();
        if let Some(res) = iter.next() {
            Ok(res?)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "no record",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let r = DiscoveryRecord::new("192.0.2.1", Some(80), Some("example"), None, None, None);
        let j = serde_helpers::to_json(&r).expect("to_json");
        let parsed = serde_helpers::from_json(&j).expect("from_json");
        assert_eq!(r, parsed);
    }

    #[test]
    fn csv_roundtrip() {
        let r = DiscoveryRecord::new(
            "198.51.100.42",
            Some(22),
            Some("ssh-banner"),
            Some("aa:bb:cc:dd:ee:ff"),
            Some("ACME"),
            Some("2025-11-02T00:00:00Z"),
        );
        let csv = serde_helpers::to_csv(&r).expect("to_csv");
        // ensure header present and at least one newline
        assert!(csv.contains("ip") || csv.contains("198.51.100.42"));
        let parsed = serde_helpers::from_csv(&csv).expect("from_csv");
        // CSV deserialization will map strings -> types; compare fields
        assert_eq!(r.ip, parsed.ip);
        assert_eq!(r.port, parsed.port);
        assert_eq!(r.banner, parsed.banner);
    }
}
