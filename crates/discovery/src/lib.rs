//! Lightweight discovery trait + simple implementations for tests/examples
//!
//! This crate provides a tiny Discovery trait and a couple of deterministic
//! example implementations meant for unit tests and documentation. The goal is
//! to provide a stable, easy-to-test contract that produces canonical
//! `formats::DiscoveryRecord` objects used across the workspace.

#[cfg(feature = "enrich")]
use enrich::vendor_from_hostname;
use formats::DiscoveryRecord;
use io::{read_netscan_csv, read_netscan_json};
use std::error::Error;
use std::path::Path;
pub mod ports;

/// A minimal discovery trait.
///
/// Inputs: list of candidate IPs or source artifacts.
/// Output: list of canonical DiscoveryRecord objects.
pub trait Discover {
    /// Perform discovery and return canonical records.
    fn discover(&self) -> Vec<DiscoveryRecord>;
}

/// Live ARP-based discoverer. Uses `netutils::cidrsniffer::scan_cidr` internally.
pub struct LiveArpDiscover {
    pub cidr: String,
    pub workers: usize,
    pub perform_probe: bool,
    /// per-lookup timeout
    pub timeout_secs: u64,
    /// enable port scanning (opt-in, off by default)
    pub portscan: bool,
    /// optional explicit port list; when None the builtin 1..=1024 is used
    pub ports: Option<Vec<u16>>,
    /// concurrency for port scanning
    pub port_concurrency: usize,
    /// per-port timeout
    pub port_timeout_secs: u64,
}

impl LiveArpDiscover {
    pub fn new<S: Into<String>>(cidr: S) -> Self {
        Self {
            cidr: cidr.into(),
            workers: 64,
            perform_probe: false, // off by default
            timeout_secs: 1,
            portscan: false,
            ports: None,
            port_concurrency: 64,
            port_timeout_secs: 1,
        }
    }

    pub fn with_workers(mut self, w: usize) -> Self {
        self.workers = w;
        self
    }

    pub fn with_probe(mut self, probe: bool) -> Self {
        self.perform_probe = probe;
        self
    }

    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable or disable port scanning (off by default)
    pub fn with_portscan(mut self, enabled: bool) -> Self {
        self.portscan = enabled;
        self
    }

    /// Set explicit ports to scan. Passing None will use the builtin 1..=1024 list.
    pub fn with_ports(mut self, ports: Option<Vec<u16>>) -> Self {
        self.ports = ports;
        self
    }

    pub fn with_port_concurrency(mut self, c: usize) -> Self {
        self.port_concurrency = c;
        self
    }

    pub fn with_port_timeout_secs(mut self, secs: u64) -> Self {
        self.port_timeout_secs = secs;
        self
    }
}

/// A simple, deterministic discoverer built from an explicit list of
/// tuples (ip, port, banner, mac, vendor, timestamp). Useful for unit tests.
pub struct SimpleDiscover {
    items: Vec<(
        String,
        Option<u16>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
}

impl SimpleDiscover {
    /// Create a new SimpleDiscover from an iterator of tuples.
    pub fn new<I>(items: I) -> Self
    where
        I: Into<
            Vec<(
                String,
                Option<u16>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            )>,
        >,
    {
        Self {
            items: items.into(),
        }
    }
}

impl Discover for LiveArpDiscover {
    fn discover(&self) -> Vec<DiscoveryRecord> {
        let timeout = std::time::Duration::from_secs(self.timeout_secs);
        match netutils::cidrsniffer::scan_cidr(
            &self.cidr,
            self.workers,
            self.perform_probe,
            timeout,
        ) {
            Ok(results) => results
                .into_iter()
                .map(|(ip, mac)| {
                    let mac_str = mac.map(|m| {
                        format!(
                            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                            m[0], m[1], m[2], m[3], m[4], m[5]
                        )
                    });
                    DiscoveryRecord::new(
                        &ip.to_string(),
                        None,
                        None,
                        mac_str.as_deref(),
                        None,
                        None,
                    )
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|r| {
                    // If portscan disabled, just return the host record
                    if !self.portscan {
                        return vec![r].into_iter();
                    }

                    // Portscan enabled: run scan_host_ports and expand per-open-port records
                    let ip_addr = match r.ip.parse::<std::net::Ipv4Addr>() {
                        Ok(a) => a,
                        Err(_) => return vec![r].into_iter(),
                    };

                    // Determine ports to scan: explicit list or builtin 1..=1024
                    let ports_vec = match &self.ports {
                        Some(v) => v.clone(),
                        None => ports::builtin_ports(),
                    };

                    let timeout = std::time::Duration::from_secs(self.port_timeout_secs);
                    let port_results = netutils::portscan::scan_host_ports(
                        ip_addr,
                        ports_vec,
                        timeout,
                        self.port_concurrency,
                    );

                    let mut out = Vec::new();
                    let mut any_open = false;
                    for p in port_results.into_iter() {
                        if p.open {
                            any_open = true;
                            let mut rec = r.clone();
                            rec.port = Some(p.port);
                            rec.banner = p.banner.clone();
                            out.push(rec);
                        }
                    }

                    if any_open {
                        out.into_iter()
                    } else {
                        // no open ports; return original host record
                        vec![r].into_iter()
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl Discover for SimpleDiscover {
    fn discover(&self) -> Vec<DiscoveryRecord> {
        self.items
            .iter()
            .map(|(ip, port, banner, mac, vendor, timestamp)| {
                // Normalization: trim and map Option<String> -> Option<&str>
                let banner_ref = banner.as_deref();
                let mac_ref = mac.as_deref();
                let vendor_ref = vendor.as_deref();
                let timestamp_ref = timestamp.as_deref();
                DiscoveryRecord::new(ip, *port, banner_ref, mac_ref, vendor_ref, timestamp_ref)
            })
            .collect()
    }
}

/// ArpSimDiscover: load legacy netscan outputs (CSV/JSON) and map them into canonical DiscoveryRecord
pub struct ArpSimDiscover {}

impl ArpSimDiscover {
    /// Load from a CSV file path (netscan-style) and return canonical DiscoveryRecord list.
    pub fn from_csv<P: AsRef<Path>>(p: P) -> Result<Vec<DiscoveryRecord>, Box<dyn Error>> {
        let mut recs = read_netscan_csv(p.as_ref().to_str().ok_or("invalid path")?)?;
        // Enrich with heuristics when enabled
        #[cfg(feature = "enrich")]
        {
            for r in recs.iter_mut() {
                if r.vendor.is_none() {
                    if let Some(b) = r.banner.as_deref() {
                        if let Some(v) = vendor_from_hostname(b) {
                            r.vendor = Some(v);
                        }
                    }
                }
            }
        }
        Ok(recs)
    }

    /// Load from a JSON file path (netscan-style) and return canonical DiscoveryRecord list.
    pub fn from_json<P: AsRef<Path>>(p: P) -> Result<Vec<DiscoveryRecord>, Box<dyn Error>> {
        let mut recs = read_netscan_json(p.as_ref().to_str().ok_or("invalid path")?)?;
        #[cfg(feature = "enrich")]
        {
            for r in recs.iter_mut() {
                if r.vendor.is_none() {
                    if let Some(b) = r.banner.as_deref() {
                        if let Some(v) = vendor_from_hostname(b) {
                            r.vendor = Some(v);
                        }
                    }
                }
            }
        }
        Ok(recs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_discover_returns_expected_records() {
        let items = vec![
            (
                "192.0.2.10".to_string(),
                Some(22),
                Some("ssh-1.0".to_string()),
                Some("aa:bb:cc:dd:ee:ff".to_string()),
                Some("ACME".to_string()),
                Some("2025-11-02T12:00:00Z".to_string()),
            ),
            ("198.51.100.5".to_string(), None, None, None, None, None),
        ];
        let s = SimpleDiscover::new(items);
        let recs = s.discover();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].ip, "192.0.2.10");
        assert_eq!(recs[0].port, Some(22));
        assert_eq!(recs[0].mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert_eq!(recs[1].ip, "198.51.100.5");
        assert_eq!(recs[1].port, None);
    }
}
