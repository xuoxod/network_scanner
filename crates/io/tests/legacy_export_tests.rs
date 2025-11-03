use formats::DiscoveryRecord;
use io::to_legacy_json;

#[test]
fn legacy_json_contains_csv_fields_and_more() {
    let r = DiscoveryRecord::new(
        "198.51.100.99",
        Some(80),
        Some("http-banner"),
        Some("de:ad:be:ef:00:01"),
        Some("VendorCo"),
        Some("2025-11-03T01:02:03Z"),
    );

    let recs = vec![r];
    let j = to_legacy_json(&recs, "arp").expect("to_legacy_json");
    let v: serde_json::Value = serde_json::from_str(&j).expect("valid json");
    assert!(v.is_array());
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let obj = &arr[0];

    // CSV fields mapped: IP, MAC, Hostname, Vendor, Timestamp
    assert_eq!(
        obj.get("IP").and_then(|x| x.as_str()).unwrap(),
        "198.51.100.99"
    );
    assert_eq!(
        obj.get("MAC").and_then(|x| x.as_str()).unwrap(),
        "de:ad:be:ef:00:01"
    );
    assert_eq!(
        obj.get("Hostname").and_then(|x| x.as_str()).unwrap(),
        "http-banner"
    );
    assert_eq!(
        obj.get("Vendor").and_then(|x| x.as_str()).unwrap(),
        "VendorCo"
    );
    assert_eq!(
        obj.get("Timestamp").and_then(|x| x.as_str()).unwrap(),
        "2025-11-03T01:02:03Z"
    );

    // Extra fields expected in legacy JSON: ports array and banners
    let ports = obj
        .get("ports")
        .and_then(|p| p.as_array())
        .expect("ports array");
    assert_eq!(ports[0].as_u64().unwrap(), 80);

    let banners = obj
        .get("banners")
        .and_then(|b| b.as_array())
        .expect("banners array");
    assert_eq!(banners[0].as_str().unwrap(), "http-banner");

    // Method and is_up fields
    assert_eq!(obj.get("Method").and_then(|m| m.as_str()).unwrap(), "arp");
    assert_eq!(obj.get("is_up").and_then(|b| b.as_bool()).unwrap(), true);
}
