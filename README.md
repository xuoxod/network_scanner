---
![logo-square](static/logo-square.svg)   ![logo](static/logo.svg)
![release](https://img.shields.io/github/v/release/xuoxod/network_scanner)

# network_scanner 🔎

![Architecture diagram](crates/discovery/docs/images/discovery-flow.svg)

![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

[Repository: xuoxod/network_scanner](https://github.com/xuoxod/network_scanner)

![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

A compact, auditable Rust toolkit for local network discovery and optional
service observation. The project is intentionally passive-first and requires
explicit opt-in for any active probing.

![Architecture diagram](crates/discovery/docs/images/discovery-flow.svg)

Key principles

1. Produce a canonical, machine-readable output format (CSV / JSON).
2. Prefer passive discovery by default; active probes are opt-in.
3. Keep library APIs small and testable; CLI is a thin wrapper.

Project layout (short)

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

Run a basic connectivity check (non-privileged):

```bash
cd /path/to/network_scanner
cargo run --manifest-path crates/netutils/Cargo.toml --bin netcheck
```

Passive discovery (no active probes):

```bash
cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --out passive.csv
```

Optional active probes and port scan — use only on networks you own or
have permission to scan:

```bash
# may require elevated privileges for some ARP probes
sudo -E cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --probe --portscan --out active.csv
```

Diagrams (per crate)

Notes

  `crates/io/data/oui.csv` for reproducible vendor lookups.
  leaking any system-specific information.

## Build and release

This repository contains several independent crates (no top-level Cargo.toml).
You can build per-crate from the repository root using `--manifest-path`, or
change directory into a crate and run `cargo` there.

Build examples (run from repository root):

- Build a crate in debug mode (default):

```bash
# build the discovery CLI in debug
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli
```

- Build release binaries:

```bash
# build the discovery CLI in release mode
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli --release
```

- Build a crate as a library in release mode:

```bash
# build the netutils crate library in release mode
cargo build --manifest-path crates/netutils/Cargo.toml --lib --release
```

- Build everything quickly (loop over crates):

```bash
for m in crates/*/Cargo.toml; do cargo build --manifest-path "$m" --release || break; done
```

Notes:

- `--manifest-path` points Cargo at the desired crate. If you prefer, `cd`
  into the crate directory and run `cargo build` without `--manifest-path`.
- `--bin <name>` builds the named binary from the crate; `--lib` builds the
  library artifact. `--release` enables optimizations.

If you want, I can add small top-level helper scripts (Makefile or a thin
shell script) to build all crates and produce a `dist/` folder.

Makefile included: run `make release-all` or `make build-all` from repo root to build crates quickly.

License

<!--
  Clean, GitHub-friendly README for network_scanner.
  This file was rewritten to remove stray code fences and duplicated content
  so it renders correctly on GitHub.
-->

![logo-square](static/logo-square.svg) ![release](https://img.shields.io/github/v/release/xuoxod/network_scanner) ![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg) ![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

# network_scanner 🔎

A compact, auditable Rust toolkit for local network discovery and optional service observation. The project is passive-first and requires explicit opt-in for any active probing.

![Architecture diagram](crates/discovery/docs/images/discovery-flow.svg)

## Key principles

1. Produce a canonical, machine-readable output format (CSV / JSON).
2. Prefer passive discovery by default; active probes are opt-in.
3. Keep library APIs small and testable; the CLI is a thin wrapper.

## Project layout (short)

```
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

## Quickstart (safe examples)

Run a basic connectivity check (non-privileged):

```bash
cd /path/to/network_scanner
cargo run --manifest-path crates/netutils/Cargo.toml --bin netcheck
```

Passive discovery (no active probes):

```bash
cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --out passive.csv
```

Optional active probes and port scan — use only on networks you own or have explicit permission to scan:

```bash
# may require elevated privileges for some ARP probes
sudo -E cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --probe --portscan --out active.csv
```

## Build and release

This repository contains several independent crates (no top-level workspace Cargo.toml). Build per-crate from the repository root using `--manifest-path`, or `cd` into a crate and run `cargo` there.

Examples:

- Build the discovery CLI in debug

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli
```

- Build the discovery CLI in release

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli --release
```

- Build netutils as a library in release

```bash
cargo build --manifest-path crates/netutils/Cargo.toml --lib --release
```

- Build all crates (quick loop)

```bash
for m in crates/*/Cargo.toml; do cargo build --manifest-path "$m" --release || break; done
```

If you prefer convenience targets, use the provided `Makefile` from the repo root:

```bash
make build-all    # debug or release depending on Makefile
make release-all  # build release for all crates
```

## Reproducible data

The project includes a canonical OUI CSV used by the IO crate at `crates/io/data/oui.csv`. Keep that in-tree for reproducible vendor lookups in tests and releases.

## License

See the `LICENSE` file at the repository root.
