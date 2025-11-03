//! Hardened OUI vendor lookup utilities for network_scanner
//!
//! This module provides a small, testable OUI mapping implementation. It can
//! be initialized from a CSV-like string (header optional) and exposes a
//! lookup function tolerant of different MAC formats.

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

// Embedded comprehensive OUI CSV shipped with this crate for reproducible builds.
static EMBEDDED_OUI_CSV: &str = include_str!("../data/oui.csv");
static OUI_MAP: OnceCell<HashMap<String, String>> = OnceCell::new();

/// Load a map from a CSV-like string. Expected rows: prefix, vendor (prefix as hex, 6 chars / 3 bytes)
pub fn load_from_str(s: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();

    // Use the csv crate to properly handle quoted fields and embedded commas.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(s.as_bytes());

    for result in rdr.records() {
        if let Ok(rec) = result {
            if rec.len() == 0 {
                continue;
            }
            // skip comments/blank first field
            let first = rec.get(0).unwrap_or("").trim();
            if first.is_empty() || first.starts_with('#') {
                continue;
            }

            // Determine which field is the assignment/prefix and which is the vendor/org
            let (maybe_prefix, vendor_field) =
                if first.to_uppercase().starts_with("MA") && rec.len() >= 3 {
                    (
                        rec.get(1).unwrap_or("").trim(),
                        rec.get(2).unwrap_or("").trim(),
                    )
                } else if rec.len() >= 2 {
                    (
                        rec.get(0).unwrap_or("").trim(),
                        rec.get(1).unwrap_or("").trim(),
                    )
                } else {
                    continue;
                };

            let key = maybe_prefix
                .replace('-', "")
                .replace(':', "")
                .to_uppercase();
            if key.len() >= 6 && key.chars().all(|c| c.is_ascii_hexdigit()) {
                m.insert(
                    key.chars().take(6).collect::<String>(),
                    vendor_field.to_string(),
                );
            }
        }
    }

    m
}

/// Initialize the default map (lazy).
fn default_map() -> &'static HashMap<String, String> {
    OUI_MAP.get_or_init(|| {
        // Try env var override first
        if let Ok(path) = std::env::var("NETWORK_SCANNER_OUI_PATH") {
            if let Ok(s) = fs::read_to_string(path) {
                return load_from_str(&s);
            }
        }
        // Try a workspace-relative path commonly used in this repo (optional)
        let candidate = Path::new("../../java/netscan/rust_backend/netutils/oui.csv");
        if candidate.exists() {
            if let Ok(s) = fs::read_to_string(candidate) {
                return load_from_str(&s);
            }
        }
        // Fallback to the embedded comprehensive CSV shipped with the crate
        load_from_str(EMBEDDED_OUI_CSV)
    })
}

/// Initialize the OUI map from an explicit file path. Returns Err on IO errors.
#[allow(dead_code)]
pub fn init_from_file<P: AsRef<Path>>(p: P) -> Result<(), Box<dyn Error>> {
    let s = fs::read_to_string(p.as_ref())?;
    let map = load_from_str(&s);
    OUI_MAP
        .set(map)
        .map_err(|_| "OUI map already initialized")?;
    Ok(())
}

/// Lookup vendor given a MAC string. Returns None if not parseable or not found.
pub fn lookup_vendor(mac: &str) -> Option<String> {
    let raw: String = mac.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if raw.len() < 6 {
        return None;
    }
    let prefix = raw[..6].to_uppercase();
    default_map().get(&prefix).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_from_str_parses_two_column_csv() {
        let csv = "000C29,\"VMware, Inc.\"\n00-16-3E,Cisco Systems";
        let m = load_from_str(csv);
        assert_eq!(m.get("000C29").map(|s| s.as_str()), Some("VMware, Inc."));
        assert_eq!(m.get("00163E").map(|s| s.as_str()), Some("Cisco Systems"));
    }

    #[test]
    fn lookup_vendor_accepts_various_mac_formats() {
        let csv = "000C29,\"VMware, Inc.\"\n00163E,Cisco Systems";
        let map = load_from_str(csv);
        // install into OnceCell temporarly for this test
        let _ = OUI_MAP.set(map);

        assert_eq!(
            lookup_vendor("00:0c:29:aa:bb:cc"),
            Some("VMware, Inc.".to_string())
        );
        assert_eq!(
            lookup_vendor("00-16-3E-01-02-03"),
            Some("Cisco Systems".to_string())
        );
        assert_eq!(
            lookup_vendor("00163E010203"),
            Some("Cisco Systems".to_string())
        );
        assert_eq!(lookup_vendor("badmac"), None);
    }

    #[test]
    fn parses_iana_ma_l_rows_and_quoted_fields() {
        let csv = "MA-L,286FB9,\"Nokia Shanghai Bell Co., Ltd.\",\"No.388 Ning Qiao Road\"\n";
        let m = load_from_str(csv);
        // key should be the first 6 hex chars of assignment
        // the parser preserves additional columns; assert the vendor contains the org name
        assert!(m
            .get("286FB9")
            .map(|s| s.contains("Nokia Shanghai Bell Co."))
            .unwrap_or(false));
    }

    #[test]
    fn preserves_vendor_commas_and_spaces() {
        let csv = "001122,\"Example, Inc.\",Some Address";
        let m = load_from_str(csv);
        // loader currently joins trailing fields (vendor + address); ensure vendor prefix preserved
        assert!(m
            .get("001122")
            .map(|s| s.starts_with("Example, Inc."))
            .unwrap_or(false));
    }

    #[test]
    fn accepts_colon_and_dash_prefixes() {
        let csv = "68:F6:3B,Amazon Technologies Inc.\n00-16-3E,Cisco Systems";
        let m = load_from_str(csv);
        assert_eq!(
            m.get("68F63B").map(|s| s.as_str()),
            Some("Amazon Technologies Inc.")
        );
        assert_eq!(m.get("00163E").map(|s| s.as_str()), Some("Cisco Systems"));
    }

    #[test]
    fn ignores_short_or_nonhex_prefixes() {
        // short assignment (too few hex digits) and non-hex characters
        let csv = "ABC,ShortVendor\nZZ:ZZ:ZZ,BadVendor";
        let m = load_from_str(csv);
        // ensure all keys (if any) are canonical 6-hex-digit prefixes
        for k in m.keys() {
            assert_eq!(k.len(), 6);
            assert!(k.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}
