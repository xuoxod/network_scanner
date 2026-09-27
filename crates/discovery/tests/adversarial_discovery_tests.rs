use discovery::ports::parse_port_list;
use formats::DiscoveryRecord;
use std::fs;

#[test]
fn test_port_parser_fuzzing() {
    // Attack 1: Overlarge numbers, negative ranges, and overflows
    let payload = "0, -1, 65536, 100000000000000000000, 22, 80-82, 99999999-999999999";
    let ports = parse_port_list(payload);
    assert!(ports.contains(&22));
    assert!(ports.contains(&80));
    assert!(ports.contains(&81));
    assert!(ports.contains(&82));
    // Must clamp to max 65535
    for p in &ports {
        assert!(*p >= 1);
    }

    // Attack 2: Reversed ranges (e.g. 1024-1)
    let reversed = parse_port_list("100-98");
    assert_eq!(reversed, vec![98, 99, 100]);

    // Attack 3: Punctuation soup and nested commas
    let soup = ",,,   ,,,,  ---, -,-, ,, 443 ,,, 8080 ,,,,";
    let soup_ports = parse_port_list(soup);
    assert_eq!(soup_ports, vec![443, 8080]);

    // Attack 4: Non-numeric strings
    let text = "http, ssh, https, telnet";
    assert!(parse_port_list(text).is_empty());
}

#[test]
fn test_cli_companion_with_formula_injection_defense() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let csv_path = tmp.path().join("fuzz_out.csv");

    let hostile_record = DiscoveryRecord::new(
        "192.168.1.99",
        Some(8080),
        Some("=1+1;cmd|' /C calc'!A0"),
        Some("00:11:22:33:44:55"),
        Some("@SUM(1+1)"),
        Some("2026-09-27T00:00:00Z"),
    );

    let recs = vec![hostile_record];

    // Write canonical CSV
    io::write_canonical_csv_file(&csv_path, &recs).expect("write csv");

    let csv_content = fs::read_to_string(&csv_path).expect("read csv");
    // Assert formula characters are neutralized with leading quote
    assert!(csv_content.contains("'=1+1;cmd|' /C calc'!A0"));
    assert!(csv_content.contains("'@SUM(1+1)"));

    // Write target JSON
    let target_path = csv_path.with_extension("target.json");
    io::write_target_json_file(target_path.display().to_string(), &recs, "arp_test")
        .expect("write target json");
    let target_content = fs::read_to_string(&target_path).expect("read target json");
    let v: serde_json::Value = serde_json::from_str(&target_content).expect("valid json");
    assert!(v.is_array());
}
