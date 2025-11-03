use formats::DiscoveryRecord;
use std::fs;

/// Simulate CLI post-processing: write CSV and then produce both the
/// target-compatible JSON and the legacy-shaped JSON companion files.
#[test]
fn cli_writes_companion_jsons() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let csv_path = tmp.path().join("out.csv");

    // prepare one discovery record
    let r = DiscoveryRecord::new(
        "127.0.0.1",
        Some(8080),
        Some("http/1.1"),
        Some("aa:bb:cc:11:22:33"),
        Some("LocalVendor"),
        Some("2025-11-03T02:03:04Z"),
    );
    let recs = vec![r];

    // write CSV as the CLI would
    let mut wtr = csv::Writer::from_path(&csv_path).expect("create csv");
    for r in recs.iter() {
        wtr.serialize(r).expect("serialize");
    }
    wtr.flush().expect("flush");

    // now emulate --json behavior: write normal .json, .target.json, and legacy .legacy.json
    let json_path = csv_path.with_extension("json");
    let target_path = csv_path.with_extension("target.json");
    let legacy_path = csv_path.with_extension("legacy.json");

    // normal pretty JSON of DiscoveryRecord list
    let s = serde_json::to_string_pretty(&recs).expect("serialize recs");
    fs::write(&json_path, &s).expect("write json");

    // target-compatible
    io::write_target_json_file(target_path.display().to_string(), &recs, "arp")
        .expect("write target json");

    // legacy-shaped
    io::write_legacy_json_file(legacy_path.display().to_string(), &recs, "arp")
        .expect("write legacy json");

    // validate files exist and basic shape
    assert!(json_path.exists());
    assert!(target_path.exists());
    assert!(legacy_path.exists());

    let t = fs::read_to_string(&target_path).expect("read target");
    let v: serde_json::Value = serde_json::from_str(&t).expect("parse target json");
    assert!(v.is_array());

    let l = fs::read_to_string(&legacy_path).expect("read legacy");
    let lv: serde_json::Value = serde_json::from_str(&l).expect("parse legacy json");
    assert!(lv.is_array());
    let obj = &lv.as_array().unwrap()[0];
    assert!(obj.get("IP").is_some());
    assert!(obj.get("ports").is_some());
    assert!(obj.get("banners").is_some());
}
