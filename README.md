# network_scanner

[Repository: xuoxod/network_scanner](https://github.com/xuoxod/network_scanner)

Purpose

This workspace provides a compact, auditable Rust toolset for LAN discovery
and lightweight service observation. It prefers passive techniques by default
and requires explicit opt-in for any active probing.

Design goals

- Canonical outputs: a stable `DiscoveryRecord` contract (CSV/JSON).
- Safety first: passive-only by default; active probing opt-in via flags.
- Library-first: CLI built on top of reusable crate APIs for testability.

Project layout (short)

```text
network_scanner/
├── crates/
│   ├── discovery/   # CLI + discovery implementations
# network_scanner

A compact, auditable Rust toolkit for local network discovery and optional
service observation. The project is intentionally passive-first and requires
explicit opt-in for any active probing.

Key principles

1. Produce a canonical, machine-readable output format (CSV / JSON).
1. Prefer passive discovery by default; active probes are opt-in.
1. Keep library APIs small and testable; CLI is a thin wrapper.

Project layout

```text
network_scanner/
├── crates/
│   ├── discovery/   # discovery implementations + discovery-cli
│   ├── io/          # data loaders and adapters (includes oui.csv)
│   ├── netutils/    # low-level helpers, netcheck, portscan
│   └── formats/     # shared DiscoveryRecord contract
├── scripts/
├── static/
├── .gitignore
└── Cargo.toml
```

Quickstart (safe, generic examples)

1. Run a basic connectivity check (non-privileged):

```bash
cd /path/to/network_scanner
cargo run --manifest-path crates/netutils/Cargo.toml --bin netcheck
```

1. Passive discovery (no active probes):

```bash
cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --out passive.csv
```

1. (Optional) Active probes and port scan — use only on networks you own or
   have permission to scan:

```bash
# may require elevated privileges for some ARP probes
sudo -E cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --probe --portscan --out active.csv
```

Diagrams (per crate)

- `crates/discovery/docs/images/discovery-flow.svg`
- `crates/discovery/docs/images/portscan-strategy.svg`
- `crates/netutils/docs/images/netutils-arch.svg`
- `crates/io/docs/images/io-arch.svg`

Notes

- The repository intentionally tracks an authoritative OUI CSV at
  `crates/io/data/oui.csv` for reproducible vendor lookups.
- All examples use generic placeholder paths and network ranges to avoid
  leaking any system-specific information.

License

See the `LICENSE` file at the repository root.
