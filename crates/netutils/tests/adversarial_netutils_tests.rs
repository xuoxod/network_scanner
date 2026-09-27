use netutils::arp::{parse_ip_neigh, parse_mac, parse_proc_net_arp};
use netutils::cidrsniffer::scan_cidr;
use netutils::portscan::normalize_banner;
use std::time::Duration;

#[test]
fn test_cidr_dos_prevention() {
    let timeout = Duration::from_millis(50);

    // 1. Massive CIDR ranges (/0 to /15) must be rejected to prevent OOM / thread starvation
    let dangerous_cidrs = vec![
        "0.0.0.0/0",
        "10.0.0.0/8",
        "172.16.0.0/12",
        "192.168.0.0/15",
    ];

    for cidr in dangerous_cidrs {
        let res = scan_cidr(cidr, 4, false, timeout);
        assert!(
            res.is_err(),
            "Dangerous CIDR {} was not rejected by safety boundary",
            cidr
        );
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("exceeds maximum supported size"));
    }

    // 2. Point-to-point /31 link must return both usable addresses (RFC 3021)
    let p2p = scan_cidr("192.168.1.10/31", 2, false, timeout).expect("/31 scan");
    assert_eq!(p2p.len(), 2);
    let ips: Vec<String> = p2p.into_iter().map(|(ip, _)| ip.to_string()).collect();
    assert!(ips.contains(&"192.168.1.10".to_string()));
    assert!(ips.contains(&"192.168.1.11".to_string()));

    // 3. Single host /32 must return exactly 1 host
    let single = scan_cidr("192.168.1.50/32", 1, false, timeout).expect("/32 scan");
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].0.to_string(), "192.168.1.50");

    // 4. Malformed and invalid CIDRs must return errors without panic
    let invalid_cidrs = vec![
        "invalid_text",
        "999.999.999.999/24",
        "192.168.1.1/33",
        "192.168.1.1/-1",
        "::1/64",
    ];
    for cidr in invalid_cidrs {
        assert!(scan_cidr(cidr, 2, false, timeout).is_err());
    }
}

#[test]
fn test_banner_ansi_escape_poisoning_defense() {
    // Attack 1: Color escapes and clear screen
    let escape_bomb = "\x1b[2J\x1b[H\x1b[31;1mCRITICAL_WARNING\x1b[0m";
    let sanitized = normalize_banner(escape_bomb);
    assert_eq!(sanitized, "CRITICAL_WARNING");
    assert!(!sanitized.contains('\x1b'));

    // Attack 2: Non-printable control characters and nulls
    let control_bomb = "SSH-2.0-OpenSSH\x00\x01\x02\x03\x04\x07\x08\tServer";
    let sanitized_control = normalize_banner(control_bomb);
    assert_eq!(sanitized_control, "SSH-2.0-OpenSSH Server");

    // Attack 3: Length explosion flood (buffer overflow defense)
    let flood = "A".repeat(100_000);
    let sanitized_flood = normalize_banner(&flood);
    assert_eq!(sanitized_flood.len(), 200); // Capped at 200 chars
}

#[test]
fn test_arp_table_corruption_resilience() {
    let corrupted_arp = r#"IP address       HW type     Flags       HW address            Mask     Device
999.999.999.999 0x1         0x2         00:11:22:33:44:55     *        eth0
192.168.1.10    0x1         0x2         NOT_A_MAC             *        eth0
-1.-2.-3.-4     0x1         0x2         00:11:22:33:44:55     *        eth0
192.168.1.20    0x1
truncated row without columns
192.168.1.30    0x1         0x2         AA:BB:CC:DD:EE:FF     *        eth0
"#;

    let parsed = parse_proc_net_arp(corrupted_arp);
    // Should parse only the single valid row (192.168.1.30)
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].0.to_string(), "192.168.1.30");
    assert_eq!(parsed[0].1, "AA:BB:CC:DD:EE:FF");

    let corrupted_neigh = r#"
malformed line
192.168.1.1 dev eth0 lladdr
192.168.1.2 dev eth0 lladdr ZZ:ZZ:ZZ:ZZ:ZZ:ZZ REACHABLE
999.999.999.999 dev eth0 lladdr 00:11:22:33:44:55 REACHABLE
192.168.1.3 dev eth0 lladdr 11:22:33:44:55:66 REACHABLE
"#;

    let parsed_neigh = parse_ip_neigh(corrupted_neigh);
    assert_eq!(parsed_neigh.len(), 1);
    assert_eq!(parsed_neigh[0].0.to_string(), "192.168.1.3");
    assert_eq!(parsed_neigh[0].1, "11:22:33:44:55:66");
}

#[test]
fn test_mac_parser_fuzzing() {
    let invalid_macs = vec![
        "",
        "00",
        "00:11:22",
        "00:11:22:33:44",
        "00:11:22:33:44:55:66",
        "00:11:22:33:44:ZZ",
        "GG:HH:II:JJ:KK:LL",
        "------------------",
        "::::::::::::::::::",
        "00-11-22-33-44",
        "00-11-22-33-44-55-66",
        "\x00\x00\x00\x00\x00\x00",
    ];

    for mac_str in invalid_macs {
        assert!(parse_mac(mac_str).is_none(), "Expected parse failure for: {}", mac_str);
    }

    // Valid formats must succeed
    assert_eq!(
        parse_mac("00:11:22:33:44:55"),
        Some([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
    );
    assert_eq!(
        parse_mac("aa-bb-cc-dd-ee-ff"),
        Some([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
    );
}
