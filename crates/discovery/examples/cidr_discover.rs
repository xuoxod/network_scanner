use discovery::{Discover, SimpleDiscover};

fn ip_range_from_cidr(cidr: &str) -> Vec<String> {
    // Only support /24 CIDR like 192.168.1.0/24 for this example
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Vec::new();
    }
    let base = parts[0];
    let octets: Vec<&str> = base.split('.').collect();
    if octets.len() != 4 {
        return Vec::new();
    }
    let prefix = format!("{}.{}.{}.", octets[0], octets[1], octets[2]);
    let mut v = Vec::new();
    for i in 1..255u8 {
        v.push(format!("{}{}", prefix, i));
    }
    v
}

fn main() {
    let cidr = "192.168.1.0/24";
    let ips = ip_range_from_cidr(cidr);
    let items: Vec<(
        String,
        Option<u16>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = ips
        .into_iter()
        .map(|ip| (ip, None, None, None, None, None))
        .collect();

    let s = SimpleDiscover::new(items);
    let recs = s.discover();
    println!("Discovered {} records for {}", recs.len(), cidr);
    for r in recs.iter().take(10) {
        println!("{}", r.ip);
    }
}
