use formats::DiscoveryRecord;
use io::to_target_json;

#[test]
fn exported_json_has_expected_shape() {
    // Prepare a sample record
    let r = DiscoveryRecord::new(
        "198.51.100.42",
        Some(22),
        Some("ssh-banner"),
        Some("aa:bb:cc:dd:ee:ff"),
        Some("ACME"),
        Some("2025-11-03T00:00:00Z"),
    );

    let recs = vec![r];

    // Serialize to target-compatible JSON (pretty-printed)
    let j = to_target_json(&recs, "portscan").expect("to_target_json");

    // Parse to serde_json::Value and assert keys/types
    let v: serde_json::Value = serde_json::from_str(&j).expect("valid json");
    assert!(v.is_array());
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let obj = &arr[0];
    assert!(obj.get("ip").is_some());
    assert!(obj.get("ports").is_some());
    assert!(obj.get("method").is_some());
    assert_eq!(obj.get("method").unwrap().as_str().unwrap(), "portscan");
    // ports should be an array containing our single port
    let ports = obj.get("ports").unwrap().as_array().unwrap();
    assert_eq!(ports[0].as_u64().unwrap(), 22);
}
