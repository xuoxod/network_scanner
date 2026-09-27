use std::env;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let timeout = Duration::from_millis(1500);

    println!("=== Network Scanner — NetCheck Connectivity Diagnostic ===");

    // 1. Detect local outbound IP
    match netutils::netcheck::local_outbound_ip() {
        Ok(ip) => println!("  Local Outbound IP: {}", ip),
        Err(e) => println!("  Local Outbound IP: [FAILED: {}]", e),
    }

    // 2. Detect default network interface and CIDR
    match netutils::iface::get_default_interface() {
        Ok(iface) => {
            let mac_str = iface
                .mac
                .map(|m| format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", m[0], m[1], m[2], m[3], m[4], m[5]))
                .unwrap_or_else(|| "none".to_string());
            let ip_str = iface
                .ipv4
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "none".to_string());
            println!(
                "  Primary Interface: {} (IPv4: {}, MAC: {}, State: {})",
                iface.name,
                ip_str,
                mac_str,
                if iface.up { "UP" } else { "DOWN" }
            );
        }
        Err(e) => println!("  Primary Interface: [FAILED: {}]", e),
    }

    match netutils::iface::get_default_cidr() {
        Ok(cidr) => println!("  Detected Local CIDR: {}", cidr),
        Err(e) => println!("  Detected Local CIDR: [UNAVAILABLE: {}]", e),
    }

    // 3. Check gateway presence (pass custom gateway as first arg or infer from CIDR/default)
    let gw_candidate = if args.len() > 1 && !args[1].starts_with('-') {
        Some(args[1].clone())
    } else {
        netutils::iface::get_default_cidr().ok().map(|cidr| {
            let octets = cidr.network().octets();
            format!("{}.{}.{}.1", octets[0], octets[1], octets[2])
        })
    };

    if let Some(ref gw) = gw_candidate {
        match netutils::netcheck::check_gateway(gw, timeout) {
            Ok(_) => println!("  Gateway Check ({}): REACHABLE (TCP 80/443)", gw),
            Err(e) => println!("  Gateway Check ({}): UNREACHABLE ({})", gw, e),
        }
    }

    // 4. Check outbound TCP connectivity (DNS / HTTP egress)
    match netutils::netcheck::check_outbound_tcp("1.1.1.1", 53, timeout) {
        Ok(_) => println!("  Outbound Egress (1.1.1.1:53): REACHABLE"),
        Err(e) => println!("  Outbound Egress (1.1.1.1:53): UNREACHABLE ({})", e),
    }

    println!("===========================================================");
}
