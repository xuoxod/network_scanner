# Discovery

![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg)
![docs.rs](https://docs.rs/discovery/badge.svg)
![crates.io](https://img.shields.io/crates/v/discovery.svg)

![discovery diagram](docs/images/discovery-flow.svg)

Discovery crate — ARP-based discovery and opt-in probing. This crate exposes a library API and a small CLI for running local network discovery. Examples use generic placeholders (do not paste system-specific values).

## Key behaviors

1. Passive ARP discovery is the default (no active probes).
2. Enable active ARP probes with `--probe` (permission required).
3. Enable TCP port scanning with `--portscan` (off by default; builtin ports cover 1..=1024). Use `--fast` for a smaller preset (~100 ports).

## Quick examples

Build the CLI in release mode:

```bash
cd /path/to/network_scanner
cargo build -p discovery --bin discovery-cli --release
```

Passive discovery (no active probes):

```bash
cargo run -p discovery --bin discovery-cli -- 10.0.0.0/24 --out results.csv
```

Opt-in active scan (only on networks you control):

```bash
# may require elevated privileges for ARP probes
sudo -E cargo run -p discovery --bin discovery-cli -- 10.0.0.0/24 --probe --portscan --out active.csv
```

## Tests

```bash
cargo test -p discovery
```

Integration test (loopback portscan):

```bash
cargo test --manifest-path crates/discovery/Cargo.toml --test portscan_integration
```

## Output formats

- CSV (default)
- JSON (enable with `--json`)

When JSON output is requested the CLI produces companion files:

- `<basename>.target.json` — neutral, target-compatible JSON (see `crates/io` helpers). This is pretty-printed and shaped for downstream consumers.
- `<basename>.legacy.json` — legacy-shaped JSON compatible with historical netscan outputs (includes `ports`, `banners`, `Method`, and `is_up`).

Use flags to control companion output:

- `--out-target <FILE>` — write the target-compatible JSON to `<FILE>`.
- `--out-legacy <FILE>` — write the legacy-shaped JSON to `<FILE>`.

## Diagrams

- `crates/discovery/docs/images/discovery-flow.svg`
- `crates/discovery/docs/images/portscan-strategy.svg`

Refer to the crate source for full API docs and examples.

## Build (quick)

From repository root, build the discovery binary in release mode:

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli --release
```

To build as a library (release):

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --lib --release
```

```bash
cargo build -p discovery --bin discovery-cli --release

```

Passive discovery (no active probes):

```bash
cargo run -p discovery --bin discovery-cli -- 10.0.0.0/24 --out results.csv
```

Opt-in active scan (only on networks you control):

```bash
# may require elevated privileges for ARP probes
sudo -E cargo run -p discovery --bin discovery-cli -- 10.0.0.0/24 --probe --portscan --out active.csv
```
