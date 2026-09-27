# Changelog

All notable changes to the `network_scanner` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-09-27

### Added
- **32,800+ Authoritative IEEE OUI Dataset**: Embedded comprehensive authoritative IEEE OUI registry (`crates/io/data/oui.csv`) containing 32,811 manufacturer records, enabling sub-microsecond, 100% offline vendor resolution for enterprise and consumer hardware without network overhead.
- **Dynamic OUI Update Engine**: Added `--update-oui` and `--oui-source <URL_OR_FILE>` CLI flags and `update_oui_cache` in `crates/io` to dynamically fetch, validate, and cache the latest IEEE assignments in the user's XDG local cache directory (`~/.local/share/network_scanner/oui.csv`).
- **Adversarial Self-Attack TDD Suites**: Created three red-team test suites (`adversarial_attack_tests.rs`, `adversarial_netutils_tests.rs`, `adversarial_discovery_tests.rs`) continuously verifying parser security, fuzzing resilience, and memory safety under hostile inputs.
- **CSV Formula Injection Defense**: Hardened canonical CSV exports against spreadsheet formula injection (`=`, `+`, `-`, `@`, `\t`, `\r`) by automatically escaping dangerous leading characters with a single-quote prefix.
- **Terminal ANSI Escape Sanitization**: Implemented robust ANSI CSI escape code stripping in service banner normalization (`normalize_banner`), neutralizing terminal hijack and log poisoning attempts.
- **CIDR DoS & Memory Exhaustion Guardrails**: Enforced maximum prefix limits (rejecting CIDR blocks wider than `/16`) in `scan_cidr` to eliminate out-of-memory crashes, with full RFC 3021 `/31` point-to-point and `/32` single-host support.
- **Kernel ARP Table Validation**: Added strict hex format verification (`parse_mac`) in `/proc/net/arp` and `ip neigh` parsers to prevent corrupted or spoofed MAC strings from polluting downstream datasets.

## [0.1.1] - 2026-09-27

### Added
- **Root Workspace Configuration**: Added top-level `Cargo.toml` with `resolver = "2"` unifying all 5 crates (`discovery`, `netutils`, `io`, `formats`, `enrich`) for root builds and tests.
- **Standalone `discovery-cli`**: Implemented CLI entry point in `crates/discovery/src/main.rs` featuring automatic primary interface CIDR auto-detection, `--fast` preset, custom `--ports`, and `--all` flag.
- **Standalone `netcheck` Diagnostic**: Created non-privileged diagnostic CLI in `crates/netutils/src/bin/netcheck.rs` inspecting local outbound IP, interface status, default CIDR, and gateway reachability.
- **Embedded OUI Vendor Lookup**: Linked discovered MAC addresses with embedded in-tree IEEE OUI table in `crates/io` to automatically populate manufacturer names.
- **Canonical CSV Writers**: Added `to_canonical_csv` and `write_canonical_csv_file` in `crates/io` guaranteeing uniform 6-column CSV rows (`ip,port,banner,mac,vendor,timestamp`).
- **Static Musl Build**: Added `make musl` target (`x86_64-unknown-linux-musl`) producing standalone static-pie binaries with zero runtime dependencies.
- **End-User VM Verification**: Verified execution and performance across Debian 12 hardware node (`xua` @ `192.168.1.160`) with clean test isolation.

### Changed
- **Modernized Documentation**: Completely overhauled root `README.md`, `crates/discovery/README.md`, and `crates/netutils/README.md` with sovereign styling, architecture diagrams, CLI references, and schema specifications.
- **Portscan Optimization**: Optimized `LiveArpDiscover` to selectively portscan only discovered live hosts, avoiding excessive timeouts across empty CIDR blocks.
- **Sanitization**: Removed hardcoded local user paths in `crates/io/tests/golden_tests.rs` and cleaned up orphaned `.bak` files.

## [0.1.0] - 2025-11-03

### Added
- Initial workspace crates: `discovery`, `netutils`, `io`, `formats`, `enrich`.
- Passive ARP discovery and opt-in TCP port scanning.
- IEEE OUI database integration (`crates/io/data/oui.csv`).
- Target-compatible and legacy JSON export companions.
- Golden file test suite and workflow CI.
