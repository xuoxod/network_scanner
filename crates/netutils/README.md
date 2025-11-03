# 🧰 netutils

![netutils architecture](docs/images/netutils-arch.svg)

Netutils — low-level network helpers

![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg) ![docs.rs](https://docs.rs/netutils/badge.svg) ![crates.io](https://img.shields.io/crates/v/netutils.svg)

This crate contains small, focused helpers used by other crates in the
workspace. Examples use generic placeholders to avoid leaking system-specific
information.

Key modules

1. `iface` — enumerate and normalize network interfaces.
1. `rawsocket` / `arp` — datalink helpers (use with care; some features may
   require elevated privileges).
1. `portscan` — TCP connect port scanning helpers (non-privileged by
   default).
1. `netcheck` — non-privileged connectivity checks and startup heuristics.

[![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg)](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml) ![docs.rs](https://docs.rs/netutils/badge.svg) ![crates.io](https://img.shields.io/crates/v/netutils.svg)

## Quick runtime check

```bash
cd /path/to/network_scanner
cargo run --manifest-path crates/netutils/Cargo.toml --bin netcheck
```

## Tests

```bash
cargo test -p netutils
```

## Build (quick)

From repository root, build the netutils library in release mode:

```bash
cargo build --manifest-path crates/netutils/Cargo.toml --lib --release
```

To build the `netcheck` binary in release mode:

```bash
cargo build --manifest-path crates/netutils/Cargo.toml --bin netcheck --release
```
