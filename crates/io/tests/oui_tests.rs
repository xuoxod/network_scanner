use std::env;
use std::fs::File;
use std::io::Read;

#[test]
fn oui_contains_expected_vmware_entry() {
    // Read the crate-local embedded CSV and assert the authoritative dump contains the expected mapping
    let path = format!("{}/data/oui.csv", env!("CARGO_MANIFEST_DIR"));
    let mut s = String::new();
    File::open(path)
        .expect("open embedded oui.csv")
        .read_to_string(&mut s)
        .expect("read embedded oui.csv");

    // Quick sanity checks: ensure the embedded CSV text contains the VMware OUI line.
    assert!(s.contains("000C29"), "embedded oui.csv must contain 000C29");
    assert!(
        s.contains("VMware"),
        "embedded oui.csv must contain VMware string"
    );
}

#[test]
fn oui_lookup_bad_mac_returns_none() {
    // Basic sanity: the library lookup helper should return None for unparseable MACs
    let vendor = io::lookup_vendor_from_oui("xyz");
    assert!(vendor.is_none());
}
