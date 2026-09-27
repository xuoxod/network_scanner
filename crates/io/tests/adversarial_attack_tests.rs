use formats::DiscoveryRecord;
use io::{load_from_str, lookup_vendor, to_canonical_csv};

#[test]
fn test_csv_formula_injection_defense() {
    let exploits = vec![
        "=cmd|' /C calc'!A0",
        "@SUM(1+1)*cmd|' /C calc'!A0",
        "+cmd /C calc",
        "-2+3+cmd|' /C calc'!A0",
        "\t=1+1",
        "\r=cmd",
    ];

    for exploit in exploits {
        let r = DiscoveryRecord::new(
            "192.168.1.100",
            Some(80),
            Some(exploit),
            Some("00:11:22:33:44:55"),
            Some(exploit),
            None,
        );

        let csv = to_canonical_csv(&[r]).expect("csv export");

        // Assert formula prefixes (=, +, -, @, \t, \r) are escaped with single quote
        assert!(
            csv.contains(&format!("'{}", exploit.trim())),
            "Formula exploit was not properly escaped in CSV: {}",
            exploit
        );
    }
}

#[test]
fn test_oui_fuzzing_resilience() {
    let flood = "A".repeat(100_000);
    let adversarial_macs: Vec<&str> = vec![
        "",
        "0",
        "00",
        "00:11",
        "ZZ:ZZ:ZZ:ZZ:ZZ:ZZ",
        "../../../../../../etc/passwd",
        "' OR '1'='1' --",
        "<script>alert(1)</script>",
        "\x00\x00\x00\x00\x00\x00",
        "\x1b[31;1mCRITICAL_ALERT\x1b[0m",
        "00:11:22\n33:44:55",
        &flood, // Giant string flood
        "00-11-22-33-44-55-66-77-88-99", // Overlong MAC
    ];

    for mac in adversarial_macs {
        // Assert lookup never panics under adversarial assault
        let res = lookup_vendor(mac);
        if mac.len() < 6 {
            assert!(res.is_none());
        }
    }
}

#[test]
fn test_oui_parser_malformed_csv_fuzzing() {
    let huge_line = "001122,".to_string() + &"A".repeat(500_000);
    let attack_payload = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        "\"UNCLOSED_QUOTE,Vendor",
        "001122,\"Embedded\0Null\",Address",
        "MA-L,,MissingPrefix,Some Address",
        "MA-L,12345678901234567890,OverlongPrefix,Vendor",
        "MA-L,ZZZZZZ,NonHexPrefix,Vendor",
        ",,,,,,ManyEmptyCommas,,,,,,,",
        huge_line,
        "'\"; DROP TABLE oui; --,MaliciousVendor"
    );

    // Assert parser handles adversarial input without panic
    let map = load_from_str(&attack_payload);
    // Should safely discard invalid records and extract valid ones if present
    assert!(!map.contains_key("ZZZZZZ"));
}
