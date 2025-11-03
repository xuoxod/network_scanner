/// Small enrichment utilities (hostname-based heuristics)

/// Given a hostname, attempt to derive a user-friendly vendor string.
/// This is heuristic-only and intended for display; it should not overwrite
/// manufacturer/vendor fields derived from OUI unless explicitly requested.
pub fn vendor_from_hostname(hostname: &str) -> Option<String> {
    let hn = hostname.to_ascii_lowercase();
    if hn.contains("mynetworksettings.com") || hn.starts_with("cr1000a") || hn.contains("fios") {
        return Some("Verizon Fios (detected)".to_string());
    }
    if hn.contains("google") || hn.contains("nest") {
        return Some("Google".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_verizon_from_cr1000a() {
        assert_eq!(
            vendor_from_hostname("CR1000A.mynetworksettings.com").as_deref(),
            Some("Verizon Fios (detected)")
        );
    }

    #[test]
    fn unknown_hostname_returns_none() {
        assert!(vendor_from_hostname("desktop.local").is_none());
    }
}
