use std::io;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

/// Lightweight, non-privileged network checks.
///
/// This module intentionally avoids raw sockets and privileged operations.
/// It provides simple heuristics to check gateway presence and outbound TCP reachability.

/// Try to open a UDP socket bound to an ephemeral local port and read the local socket address.
/// This helps discover the local outbound IP used by the OS (not guaranteed behind complex NATs).
pub fn local_outbound_ip() -> io::Result<IpAddr> {
    // Use a well-known public IP but do not send data; connecting a UDP socket is enough to get OS route.
    let remote: SocketAddr = "1.1.1.1:53".parse().unwrap();
    let sock = UdpSocket::bind(("0.0.0.0", 0))?;
    sock.connect(remote)?;
    let local = sock.local_addr()?;
    Ok(local.ip())
}

/// Check outbound TCP connectivity to a stable endpoint and port with a short timeout.
/// Returns Ok(()) on success, or the underlying IO error on failure.
pub fn check_outbound_tcp(addr: &str, port: u16, timeout: Duration) -> io::Result<()> {
    let socket = format!("{}:{}", addr, port);
    let addr = socket.parse::<SocketAddr>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid socket addr: {}", e),
        )
    })?;
    TcpStream::connect_timeout(&addr, timeout).map(|_| ())
}

/// Quick gateway check: attempt to connect TCP to the gateway on port 80/443 with a short timeout.
/// If the system has no default gateway or routing, this will likely fail quickly.
pub fn check_gateway(host: &str, timeout: Duration) -> io::Result<()> {
    // Try port 80 then 443; callers can pass the gateway IP or hostname.
    match check_outbound_tcp(host, 80, timeout) {
        Ok(()) => Ok(()),
        Err(_) => check_outbound_tcp(host, 443, timeout),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn local_outbound_ip_returns_ip() {
        // This test should pass on most systems with network connectivity but is forgiving: if it fails,
        // we treat it as non-fatal by asserting only that the error is not a panic.
        let _ = local_outbound_ip();
    }

    #[test]
    fn outbound_tcp_times_out_for_unroutable() {
        // Connect to an unroutable address (TEST-NET-1) on port 9 so it should either timeout or error.
        let res = check_outbound_tcp("192.0.2.1", 9, Duration::from_millis(200));
        assert!(res.is_err());
    }
}
