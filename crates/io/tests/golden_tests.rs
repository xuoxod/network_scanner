use std::fs::read_to_string;
use std::path::Path;

use io::{read_netscan_csv, read_netscan_json};
use serde_json::Value;

fn normalize_json(s: &str) -> Value {
    serde_json::from_str(s).expect("valid json")
}

#[test]
fn csv_against_golden() {
    let sample = "/home/emhcet/Downloads/d-nodes/discovered_hosts.csv";
    if !Path::new(sample).exists() {
        eprintln!("skipping csv golden test (sample missing)");
        return;
    }
    let mapped = read_netscan_csv(sample).expect("read csv");
    let produced = serde_json::to_string_pretty(&mapped).expect("serialize produced");

    let golden_path = "tests/golden/discovered_hosts.csv.golden.json";
    let golden = read_to_string(golden_path)
        .expect("golden file exists - generate with `cargo run --bin generate_golden` if missing");

    let a = normalize_json(&produced);
    let b = normalize_json(&golden);
    assert_eq!(a, b, "CSV mapping does not match golden file");
}

#[test]
fn json_against_golden() {
    let sample = "/home/emhcet/Downloads/d-nodes/discovered_hosts.json";
    if !Path::new(sample).exists() {
        eprintln!("skipping json golden test (sample missing)");
        return;
    }
    let mapped = read_netscan_json(sample).expect("read json");
    let produced = serde_json::to_string_pretty(&mapped).expect("serialize produced");

    let golden_path = "tests/golden/discovered_hosts.json.golden.json";
    let golden = read_to_string(golden_path)
        .expect("golden file exists - generate with `cargo run --bin generate_golden` if missing");

    let a = normalize_json(&produced);
    let b = normalize_json(&golden);
    assert_eq!(a, b, "JSON mapping does not match golden file");
}
