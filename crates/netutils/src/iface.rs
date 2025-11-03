use ipnetwork::{IpNetwork, Ipv4Network};
use std::fmt;
use std::net::Ipv4Addr;

/// Represents a network interface on the system.
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub index: u32,
    pub mac: Option<[u8; 6]>,
    pub ipv4: Option<Ipv4Addr>,
    pub up: bool,
}

#[derive(Debug)]
pub enum IfaceError {
    NotFound,
    NoUpInterface,
    Io(std::io::Error),
    Platform(String),
    PermissionDenied(String),
    InvalidInterface(String),
    Other(String), // Added for custom errors
}

impl fmt::Display for IfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaceError::NotFound => write!(f, "Interface not found"),
            IfaceError::NoUpInterface => write!(f, "No up interface with IPv4"),
            IfaceError::Io(e) => write!(f, "IO error: {}", e),
            IfaceError::Platform(s) => write!(f, "Platform error: {}", s),
            IfaceError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            IfaceError::InvalidInterface(s) => write!(f, "Invalid interface: {}", s),
            IfaceError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl std::error::Error for IfaceError {}

/// Returns the default network's CIDR (IPv4Network) for the primary interface.
/// Falls back to /24 if we can't determine a mask.
pub fn get_default_cidr() -> Result<Ipv4Network, IfaceError> {
    let iface = get_default_interface()?;
    let ipv4 = iface.ipv4.ok_or(IfaceError::NoUpInterface)?;
    // Try to get netmask from pnet_datalink
    let interfaces = pnet_datalink::interfaces();
    for i in interfaces {
        if i.name == iface.name {
            for ip in i.ips {
                if let IpNetwork::V4(net) = ip {
                    if net.ip() == ipv4 {
                        return Ok(net);
                    }
                }
            }
        }
    }
    // Fallback: /24
    Ok(Ipv4Network::new(ipv4, 24).map_err(|_| IfaceError::NoUpInterface)?)
}

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Command;

/// Returns the default gateway IPv4 address by parsing /proc/net/route (Linux only).
pub fn get_default_gateway_ipv4() -> Option<Ipv4Addr> {
    let file = fs::File::open("/proc/net/route").ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines().skip(1) {
        if let Ok(line) = line {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 3 && fields[1] == "00000000" {
                if let Ok(gw_hex) = u32::from_str_radix(fields[2], 16) {
                    let gw_bytes = gw_hex.to_le_bytes();
                    return Some(Ipv4Addr::new(
                        gw_bytes[0],
                        gw_bytes[1],
                        gw_bytes[2],
                        gw_bytes[3],
                    ));
                }
            }
        }
    }
    None
}

/// Returns the MAC address for a given IPv4 address from the ARP table (Linux only).
pub fn get_mac_for_ipv4(ip: Ipv4Addr) -> Option<[u8; 6]> {
    // Prefer `ip neigh` output which is more likely to be present on modern systems.
    if let Ok(output) = Command::new("ip").args(["neigh"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(&ip.to_string()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(idx) = parts.iter().position(|&s| s == "lladdr") {
                    if let Some(mac_str) = parts.get(idx + 1) {
                        let mac_bytes: Vec<u8> = mac_str
                            .split(':')
                            .filter_map(|b| u8::from_str_radix(b, 16).ok())
                            .collect();
                        if mac_bytes.len() == 6 {
                            let mut mac = [0u8; 6];
                            mac.copy_from_slice(&mac_bytes);
                            return Some(mac);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Returns a list of all network interfaces on the system.
pub fn list_interfaces() -> Result<Vec<NetworkInterface>, IfaceError> {
    // Use pnet_datalink for cross-platform interface listing
    let ifaces = pnet_datalink::interfaces();
    let result = ifaces
        .into_iter()
        .map(|iface| NetworkInterface {
            name: iface.name.clone(),
            index: iface.index,
            mac: iface.mac.map(|m| m.octets()),
            ipv4: iface.ips.iter().find_map(|ip| match ip {
                IpNetwork::V4(ipv4) => Some(ipv4.ip()),
                _ => None,
            }),
            up: iface.is_up(),
        })
        .collect();
    Ok(result)
}

/// Attempts to find the system's default (primary) network interface that is up and has an IPv4 address.
pub fn get_default_interface() -> Result<NetworkInterface, IfaceError> {
    let interfaces = list_interfaces()?;
    // Prefer non-loopback, up, with IPv4
    interfaces
        .into_iter()
        .find(|iface| iface.up && iface.ipv4.is_some() && !iface.name.starts_with("lo"))
        .ok_or(IfaceError::NoUpInterface)
}

/// Finds an interface by name.
pub fn get_interface_by_name(name: &str) -> Result<NetworkInterface, IfaceError> {
    let interfaces = list_interfaces()?;
    interfaces
        .into_iter()
        .find(|iface| iface.name == name)
        .ok_or(IfaceError::NotFound)
}

/// Finds an interface by index.
pub fn get_interface_by_index(index: u32) -> Result<NetworkInterface, IfaceError> {
    let interfaces = list_interfaces()?;
    interfaces
        .into_iter()
        .find(|iface| iface.index == index)
        .ok_or(IfaceError::NotFound)
}

/// Finds an interface by MAC address.
pub fn get_interface_by_mac(mac: [u8; 6]) -> Result<NetworkInterface, IfaceError> {
    let interfaces = list_interfaces()?;
    interfaces
        .into_iter()
        .find(|iface| iface.mac == Some(mac))
        .ok_or(IfaceError::NotFound)
}

/// Finds an interface by IPv4 address.
pub fn get_interface_by_ipv4(ipv4: Ipv4Addr) -> Result<NetworkInterface, IfaceError> {
    let interfaces = list_interfaces()?;
    interfaces
        .into_iter()
        .find(|iface| iface.ipv4 == Some(ipv4))
        .ok_or(IfaceError::NotFound)
}

/// Finds an interface by name or index.
pub fn get_interface_by_name_or_index(
    name: Option<&str>,
    index: Option<u32>,
) -> Result<NetworkInterface, IfaceError> {
    if let Some(name) = name {
        get_interface_by_name(name)
    } else if let Some(index) = index {
        get_interface_by_index(index)
    } else {
        Err(IfaceError::NotFound)
    }
}

/// Finds an interface by MAC address or IPv4 address.
pub fn get_interface_by_mac_or_ipv4(
    mac: Option<[u8; 6]>,
    ipv4: Option<Ipv4Addr>,
) -> Result<NetworkInterface, IfaceError> {
    if let Some(mac) = mac {
        get_interface_by_mac(mac)
    } else if let Some(ipv4) = ipv4 {
        get_interface_by_ipv4(ipv4)
    } else {
        Err(IfaceError::NotFound)
    }
}

/// Finds an interface by name, index, MAC address, or IPv4 address.
pub fn get_interface_by_name_index_mac_ipv4(
    name: Option<&str>,
    index: Option<u32>,
    mac: Option<[u8; 6]>,
    ipv4: Option<Ipv4Addr>,
) -> Result<NetworkInterface, IfaceError> {
    if let Some(name) = name {
        get_interface_by_name(name)
    } else if let Some(index) = index {
        get_interface_by_index(index)
    } else if let Some(mac) = mac {
        get_interface_by_mac(mac)
    } else if let Some(ipv4) = ipv4 {
        get_interface_by_ipv4(ipv4)
    } else {
        Err(IfaceError::NotFound)
    }
}

/// Returns true if the interface is NOT managed by a DHCP client (Linux heuristics).
pub fn is_interface_unmanaged(interface: &str) -> Result<bool, IfaceError> {
    // Linux: Check for dhclient, systemd-networkd, NetworkManager leases
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Check common lease files
        let lease_paths = [
            format!("/run/systemd/netif/leases/{}", interface),
            format!("/var/lib/dhcp/dhclient.{}.leases", interface),
            format!("/var/lib/NetworkManager/dhclient-{}.lease", interface),
        ];
        for path in &lease_paths {
            if fs::metadata(path).is_ok() {
                return Ok(false);
            }
        }
        // Optionally, check with nmcli
        if let Ok(output) = std::process::Command::new("nmcli")
            .args(["device", "show", interface])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("DHCP4") {
                return Ok(false);
            }
        }
        Ok(true)
    }
    #[cfg(not(target_os = "linux"))]
    {
        // TODO: Implement for other platforms
        Ok(true)
    }
}

pub fn resolve_iface_name(interface: &Option<String>) -> String {
    match interface.as_deref() {
        Some(name) => name.to_string(),
        None => {
            get_default_interface()
                .expect("No default interface found")
                .name
        }
    }
}

/// Tests: exercise common, non-destructive behaviors. These tests are intentionally
/// conservative (they only assert presence of interfaces and roundtrip queries).
#[cfg(test)]
mod tests {
    use super::*;
    // Ipv4Addr already imported where needed; remove duplicate import to silence warning.

    #[test]
    fn test_list_interfaces_not_empty() {
        let interfaces = list_interfaces().expect("Should list interfaces");
        assert!(
            !interfaces.is_empty(),
            "There should be at least one interface"
        );
    }

    #[test]
    fn test_get_default_interface_is_up_and_has_ipv4() {
        let iface = get_default_interface().expect("Should find a default interface");
        assert!(iface.up, "Default interface should be up");
        assert!(
            iface.ipv4.is_some(),
            "Default interface should have an IPv4 address"
        );
        assert!(
            !iface.name.starts_with("lo"),
            "Default interface should not be loopback"
        );
    }

    #[test]
    fn test_get_interface_by_name_roundtrip() {
        let interfaces = list_interfaces().expect("Should list interfaces");
        let iface = interfaces
            .iter()
            .find(|iface| iface.up && iface.ipv4.is_some() && !iface.name.starts_with("lo"))
            .expect("Should have at least one up, non-loopback interface with IPv4");
        let by_name = get_interface_by_name(&iface.name).expect("Should find interface by name");
        assert_eq!(iface.name, by_name.name);
        assert_eq!(iface.index, by_name.index);
    }

    #[test]
    fn test_get_interface_by_index_roundtrip() {
        let interfaces = list_interfaces().expect("Should list interfaces");
        let iface = interfaces
            .iter()
            .find(|iface| iface.up && iface.ipv4.is_some() && !iface.name.starts_with("lo"))
            .expect("Should have at least one up, non-loopback interface with IPv4");
        let by_index = get_interface_by_index(iface.index).expect("Should find interface by index");
        assert_eq!(iface.name, by_index.name);
        assert_eq!(iface.index, by_index.index);
    }

    #[test]
    fn test_get_interface_by_mac_or_ipv4() {
        let interfaces = list_interfaces().expect("Should list interfaces");
        let iface = interfaces.iter().find(|iface| {
            iface.up && iface.ipv4.is_some() && iface.mac.is_some() && !iface.name.starts_with("lo")
        });
        if let Some(iface) = iface {
            let by_mac = get_interface_by_mac_or_ipv4(iface.mac, None).expect("Should find by MAC");
            assert_eq!(iface.mac, by_mac.mac);
            let by_ipv4 =
                get_interface_by_mac_or_ipv4(None, iface.ipv4).expect("Should find by IPv4");
            assert_eq!(iface.ipv4, by_ipv4.ipv4);
        }
    }

    #[test]
    fn test_get_interface_by_name_index_mac_ipv4() {
        let interfaces = list_interfaces().expect("Should list interfaces");
        let iface = interfaces.iter().find(|iface| {
            iface.up && iface.ipv4.is_some() && iface.mac.is_some() && !iface.name.starts_with("lo")
        });
        if let Some(iface) = iface {
            let by_name = get_interface_by_name_index_mac_ipv4(Some(&iface.name), None, None, None)
                .expect("Should find by name");
            assert_eq!(iface.name, by_name.name);

            let by_index =
                get_interface_by_name_index_mac_ipv4(None, Some(iface.index), None, None)
                    .expect("Should find by index");
            assert_eq!(iface.index, by_index.index);

            let by_mac = get_interface_by_name_index_mac_ipv4(None, None, iface.mac, None)
                .expect("Should find by MAC");
            assert_eq!(iface.mac, by_mac.mac);

            let by_ipv4 = get_interface_by_name_index_mac_ipv4(None, None, None, iface.ipv4)
                .expect("Should find by IPv4");
            assert_eq!(iface.ipv4, by_ipv4.ipv4);
        }
    }

    #[test]
    fn test_get_interface_by_name_not_found() {
        let result = get_interface_by_name("definitely_not_a_real_interface_name_12345");
        assert!(matches!(result, Err(IfaceError::NotFound)));
    }
}
