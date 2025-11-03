use discovery::ArpSimDiscover;
use std::path::Path;

#[test]
fn load_golden_csv_via_arp_sim() {
    let p = Path::new("../io/tests/golden/discovered_hosts.csv.golden.json");
    if !p.exists() {
        eprintln!("Skipping test: golden CSV json not present: {:?}", p);
        return;
    }
    // The golden file is JSON of canonical DiscoveryRecord; ArpSimDiscover::from_csv expects netscan CSV
    // so instead we test from_json path
    let json_path = Path::new("../io/tests/golden/discovered_hosts.json.golden.json");
    if !json_path.exists() {
        eprintln!("Skipping test: golden JSON not present: {:?}", json_path);
        return;
    }
    let recs = ArpSimDiscover::from_json(json_path).expect("read json golden");
    assert!(!recs.is_empty());
}
