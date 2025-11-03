/// Return the builtin thorough ports list (1..=1024).
pub fn builtin_ports() -> Vec<u16> {
    (1u16..=1024u16).collect()
}

/// Fast preset: top ~100 commonly used ports for a quick scan.
pub fn fast_ports() -> Vec<u16> {
    vec![
        20, 21, 22, 23, 25, 37, 53, 67, 68, 69, 79, 80, 81, 88, 103, 110, 111, 113, 119, 123, 135,
        137, 138, 139, 143, 161, 162, 179, 199, 389, 443, 445, 465, 514, 515, 540, 554, 587, 593,
        631, 636, 666, 989, 990, 993, 995, 1025, 1080, 1194, 1433, 1434, 1521, 1720, 1723, 1863,
        1900, 2049, 2082, 2083, 2086, 2087, 2095, 2096, 2121, 2222, 2302, 2483, 2484, 25565, 2601,
        2717, 3000, 3128, 3306, 3389, 3478, 3690, 3702, 3986, 4000, 4500, 4899, 5000, 5001, 5060,
        5061, 5222, 5232, 5432, 5555, 5601, 5900, 5984, 6379, 6667, 6881, 6969, 7000, 7199, 8000,
        8008, 8080, 8081, 8443, 8888, 9000, 9100, 9200, 9300, 10000, 27017,
    ]
}

/// Parse a port list string like "22,80,443,8000-8100" into Vec<u16>.
/// This parser is forgiving: it will skip invalid tokens, clamp to 1..=65535,
/// accept ranges in any order, deduplicate and sort the result.
/// If no valid ports are found, an empty Vec is returned.
pub fn parse_port_list(s: &str) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for token in s.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if let Some(idx) = token.find('-') {
            // range syntax a-b; be forgiving about whitespace
            let a = token[..idx].trim();
            let b = token[idx + 1..].trim();
            if let (Ok(mut start), Ok(mut end)) = (a.parse::<i64>(), b.parse::<i64>()) {
                // clamp to valid port range and normalize order
                if start < 1 {
                    start = 1
                };
                if end < 1 {
                    end = 1
                };
                if start > 65535 {
                    start = 65535
                };
                if end > 65535 {
                    end = 65535
                };
                if start <= end {
                    for p in start as u16..=end as u16 {
                        out.push(p);
                    }
                } else {
                    // reversed range e.g. 1024-1 -> interpret as 1..=1024
                    for p in end as u16..=start as u16 {
                        out.push(p);
                    }
                }
            } else {
                // skip invalid range tokens
                continue;
            }
        } else {
            // single port token
            if let Ok(mut p) = token.parse::<i64>() {
                if p < 1 {
                    continue;
                }
                if p > 65535 {
                    p = 65535;
                }
                out.push(p as u16);
            } else {
                // skip invalid token
                continue;
            }
        }
    }

    // dedupe and sort
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_list() {
        let v = parse_port_list("22,80,443");
        assert_eq!(v, vec![22, 80, 443]);
    }

    #[test]
    fn parse_range() {
        let v = parse_port_list("8000-8002");
        assert_eq!(v, vec![8000, 8001, 8002]);
    }

    #[test]
    fn parse_mixed() {
        let v = parse_port_list("22,1000-1002,22");
        assert_eq!(v, vec![22, 1000, 1001, 1002]);
    }

    #[test]
    fn parse_invalid_tokens_are_ignored() {
        let v = parse_port_list("22,abc,70000,10-8, - , 443");
        // 70000 clamps to 65535, but 65535 is valid; 10-8 -> 8..=10
        assert!(v.contains(&22));
        assert!(v.contains(&443));
        assert!(v.contains(&8));
        assert!(v.contains(&9));
        assert!(v.contains(&10));
        assert!(v.contains(&65535));
    }

    #[test]
    fn empty_or_all_invalid_returns_empty() {
        let v = parse_port_list("");
        assert!(v.is_empty());
        let v2 = parse_port_list("foo,bar,-");
        assert!(v2.is_empty());
    }
}
