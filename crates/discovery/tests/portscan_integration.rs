use netutils::portscan;
use std::net::{Ipv4Addr, TcpListener};
use std::thread;
use std::time::Duration;

#[test]
fn detect_local_tcp_listener() {
    // bind an ephemeral listener that writes a banner
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        if let Ok((mut s, _)) = listener.accept() {
            use std::io::Write;
            let _ = s.write_all(b"CLI-TEST\n");
            // keep connection briefly
            thread::sleep(Duration::from_millis(200));
        }
    });

    let ip = match addr.ip() {
        std::net::IpAddr::V4(v4) => v4,
        _ => panic!("expected ipv4 local addr"),
    };
    let ports = vec![addr.port()];
    let res = portscan::scan_host_ports(ip, ports, Duration::from_secs(2), 2);
    assert_eq!(res.len(), 1);
    assert!(res[0].open);
    assert_eq!(res[0].port, addr.port());
    assert!(res[0]
        .banner
        .as_deref()
        .map(|s| s.contains("CLI-TEST"))
        .unwrap_or(false));
}
