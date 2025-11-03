//! IO adapters for legacy netscan JSON/CSV into canonical `formats::DiscoveryRecord`

use std::error::Error;
use std::fs::File;
use std::io::Read;

use formats::DiscoveryRecord;
mod oui;
pub use oui::lookup_vendor as lookup_vendor_from_oui;

/// Read a netscan-style JSON file and map to canonical DiscoveryRecord list.
pub fn read_netscan_json<P: AsRef<str>>(path: P) -> Result<Vec<DiscoveryRecord>, Box<dyn Error>> {
    let path = path.as_ref();
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    let v: serde_json::Value = serde_json::from_str(&s)?;
    let arr = v
        .as_array()
        .ok_or_else(|| "expected top-level array in netscan json")?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let ip = item
            .get("IP")
            .and_then(|x| x.as_str())
            .or_else(|| item.get("ip").and_then(|x| x.as_str()))
            .ok_or("missing IP")?;
        // prefer explicit ports array if present
        let port = item
            .get("ports")
            .and_then(|p| p.as_array())
            .and_then(|a| a.get(0))
            .and_then(|n| n.as_u64())
            .map(|n| n as u16);
        // prefer Hostname or first banner
        let banner = item
            .get("Hostname")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                item.get("banners")
                    .and_then(|b| b.as_array())
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        // optional fields commonly present in netscan outputs
        let mac = item
            .get("MAC")
            .and_then(|x| x.as_str())
            .or_else(|| item.get("mac").and_then(|x| x.as_str()));
        let vendor = item
            .get("Vendor")
            .and_then(|x| x.as_str())
            .or_else(|| item.get("vendor").and_then(|x| x.as_str()));
        let timestamp = item
            .get("Timestamp")
            .and_then(|x| x.as_str())
            .or_else(|| item.get("timestamp").and_then(|x| x.as_str()))
            .or_else(|| item.get("time").and_then(|x| x.as_str()));

        out.push(DiscoveryRecord::new(
            ip,
            port,
            banner.as_deref(),
            mac,
            vendor,
            timestamp,
        ));
    }
    Ok(out)
}

/// Export a list of `DiscoveryRecord` as a JSON array compatible with the
/// Target-compatible JSON exporter. Produces pretty-printed JSON arrays that
/// are intended to be ingested by external consumers. The naming here is
/// intentionally neutral to avoid coupling to any downstream product names.
pub fn to_target_json(
    records: &[DiscoveryRecord],
    default_method: &str,
) -> Result<String, Box<dyn Error>> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct GoDevice<'a> {
        ip: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        mac: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hostname: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        vendor: Option<&'a str>,
        method: &'a str,
        ports: Vec<u16>,
        is_up: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<&'a str>,
    }

    let mut out = Vec::with_capacity(records.len());
    for r in records {
        let ports = r.port.map(|p| vec![p]).unwrap_or_default();
        let hostname = r.banner.as_deref();
        let dev = GoDevice {
            ip: &r.ip,
            mac: r.mac.as_deref(),
            hostname,
            vendor: r.vendor.as_deref(),
            method: default_method,
            ports,
            is_up: true,
            timestamp: r.timestamp.as_deref(),
        };
        out.push(dev);
    }

    Ok(serde_json::to_string_pretty(&out)?)
}

/// Convenience: write target-compatible JSON to a file path.
pub fn write_target_json_file<P: AsRef<str>>(
    path: P,
    records: &[DiscoveryRecord],
    default_method: &str,
) -> Result<(), Box<dyn Error>> {
    let s = to_target_json(records, default_method)?;
    std::fs::write(path.as_ref(), s)?;
    Ok(())
}

/// Export a list of `DiscoveryRecord` in a legacy netscan-shaped JSON format.
/// This retains all CSV-provided fields and adds richer per-device details
/// (ports array, banners array, method, is_up). The goal is a drop-in
/// replacement for legacy consumers while keeping the exporter code in `io`.
pub fn to_legacy_json(
    records: &[DiscoveryRecord],
    default_method: &str,
) -> Result<String, Box<dyn Error>> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct LegacyDevice<'a> {
        // Use snake_case field names to satisfy Rust naming lint rules,
        // but rename them to the legacy keys when serializing.
        #[serde(rename = "IP")]
        ip: &'a str,
        #[serde(rename = "MAC", skip_serializing_if = "Option::is_none")]
        mac: Option<&'a str>,
        #[serde(rename = "Hostname", skip_serializing_if = "Option::is_none")]
        hostname: Option<&'a str>,
        #[serde(rename = "Vendor", skip_serializing_if = "Option::is_none")]
        vendor: Option<&'a str>,
        #[serde(rename = "Timestamp", skip_serializing_if = "Option::is_none")]
        timestamp: Option<&'a str>,
        // richer fields not present in minimal CSV
        ports: Vec<u16>,
        banners: Vec<&'a str>,
        #[serde(rename = "is_up")]
        is_up: bool,
        #[serde(rename = "Method")]
        method: &'a str,
    }

    let mut out = Vec::with_capacity(records.len());
    for r in records {
        let ports = r.port.map(|p| vec![p]).unwrap_or_default();
        let mut banners = Vec::new();
        if let Some(b) = r.banner.as_deref() {
            if !b.is_empty() {
                banners.push(b);
            }
        }

        let dev = LegacyDevice {
            ip: &r.ip,
            mac: r.mac.as_deref(),
            hostname: r.banner.as_deref(),
            vendor: r.vendor.as_deref(),
            timestamp: r.timestamp.as_deref(),
            ports,
            banners,
            is_up: true,
            method: default_method,
        };
        out.push(dev);
    }

    Ok(serde_json::to_string_pretty(&out)?)
}

/// Convenience: write legacy-shaped JSON to a file path.
pub fn write_legacy_json_file<P: AsRef<str>>(
    path: P,
    records: &[DiscoveryRecord],
    default_method: &str,
) -> Result<(), Box<dyn Error>> {
    let s = to_legacy_json(records, default_method)?;
    std::fs::write(path.as_ref(), s)?;
    Ok(())
}

/// Read a netscan-style CSV file and map to canonical DiscoveryRecord list.
/// Expected CSV headers (common netscan): Timestamp,IP,MAC,Hostname,Vendor,OS
pub fn read_netscan_csv<P: AsRef<str>>(path: P) -> Result<Vec<DiscoveryRecord>, Box<dyn Error>> {
    let path = path.as_ref();
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();

    // Use header names to find columns so CSVs with different column order work.
    // Expected headers include: Timestamp,IP,MAC,Hostname,Vendor,OS
    let headers = rdr.headers()?.clone();
    let find = |names: &[&str]| {
        names
            .iter()
            .filter_map(|n| headers.iter().position(|h| h.eq_ignore_ascii_case(n)))
            .next()
    };

    let ip_idx_default = find(&["ip", "IP"]).or(Some(1)).unwrap_or(1);
    let mac_idx_default = find(&["mac", "MAC"]);
    let ts_idx_default = find(&["timestamp", "time", "Timestamp"]);
    let host_idx_default = find(&["hostname", "host", "Host"]);
    let vendor_idx_default = find(&["vendor", "Vendor"]);

    for result in rdr.records() {
        let rec = result?;

        let ip = rec
            .get(ip_idx_default)
            .ok_or("missing IP column")?
            .trim()
            .to_string();

        let hostname = host_idx_default.and_then(|i| rec.get(i)).and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s.trim())
            }
        });

        let mac = mac_idx_default.and_then(|i| rec.get(i)).and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        });

        let vendor = vendor_idx_default.and_then(|i| rec.get(i)).and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        });

        let timestamp = ts_idx_default.and_then(|i| rec.get(i)).and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        });

        // No port info in this CSV; leave None
        out.push(DiscoveryRecord::new(
            &ip, None, hostname, mac, vendor, timestamp,
        ));
    }

    Ok(out)
}
