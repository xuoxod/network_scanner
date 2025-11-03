use discovery::Discover;
use discovery::LiveArpDiscover;
use formats::DiscoveryRecord;
use std::env;
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;

fn usage(prog: &str) {
    eprintln!("Usage: {} <cidr> [--probe] [--portscan] [--out file.csv] [--json] [--concurrency N] [--timeout secs]", prog);
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let prog = args
        .get(0)
        .cloned()
        .unwrap_or_else(|| "live_arpscan".into());
    if args.len() < 2 {
        usage(&prog);
        return;
    }

    let cidr = args[1].clone();
    let mut perform_probe = false;
    let mut do_portscan = false;
    let mut out_csv: PathBuf = PathBuf::from("discovery_results.csv");
    let mut write_json = false;
    let mut concurrency = 64usize;
    let mut timeout_secs = 1u64;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--probe" => {
                perform_probe = true;
                i += 1;
            }
            "--portscan" => {
                do_portscan = true;
                i += 1;
            }
            "--out" => {
                if i + 1 < args.len() {
                    out_csv = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    usage(&prog);
                    return;
                }
            }
            "--json" => {
                write_json = true;
                i += 1;
            }
            "--concurrency" => {
                if i + 1 < args.len() {
                    concurrency = args[i + 1].parse().unwrap_or(concurrency);
                    i += 2;
                } else {
                    usage(&prog);
                    return;
                }
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    timeout_secs = args[i + 1].parse().unwrap_or(timeout_secs);
                    i += 2;
                } else {
                    usage(&prog);
                    return;
                }
            }
            _ => {
                eprintln!("Unknown arg: {}", args[i]);
                usage(&prog);
                return;
            }
        }
    }

    let mut discover = LiveArpDiscover::new(cidr)
        .with_workers(concurrency)
        .with_probe(perform_probe)
        .with_timeout_secs(timeout_secs);

    let records: Vec<DiscoveryRecord> = discover.discover();

    // Optionally run portscan per host (opt-in). Default built-in ports are 1..=1024
    let mut final_records = Vec::new();
    if do_portscan {
        eprintln!("Performing portscan on discovered hosts (this may take a while)...");
        for r in records.iter() {
            let ip: Ipv4Addr = r.ip.parse().unwrap_or(Ipv4Addr::UNSPECIFIED);
            if ip == Ipv4Addr::UNSPECIFIED {
                continue;
            }
            // ports 1..=1024
            let ports: Vec<u16> = (1u16..=1024u16).collect();
            let port_results = netutils::portscan::scan_host_ports(
                ip,
                ports,
                Duration::from_secs(timeout_secs),
                concurrency,
            );
            if port_results.is_empty() {
                final_records.push(r.clone());
            } else {
                for p in port_results {
                    let mut rec = r.clone();
                    rec.port = Some(p.port);
                    rec.banner = p.banner.clone();
                    final_records.push(rec);
                }
            }
        }
    } else {
        final_records = records;
    }

    // Write CSV by default
    if let Ok(mut w) = File::create(&out_csv) {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        for r in final_records.iter() {
            let _ = wtr.serialize(r);
        }
        let _ = wtr.flush();
        if let Ok(bytes) = wtr.into_inner() {
            let _ = w.write_all(&bytes);
            println!("Wrote CSV to {}", out_csv.display());
        }
    } else {
        eprintln!("Failed to open output file {}", out_csv.display());
    }

    if write_json {
        let json_out = out_csv.with_extension("json");
        if let Ok(mut f) = File::create(&json_out) {
            if let Ok(s) = serde_json::to_string(&final_records) {
                let _ = f.write_all(s.as_bytes());
                println!("Wrote JSON to {}", json_out.display());
            }
        }
    }
}
