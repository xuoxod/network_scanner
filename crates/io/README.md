# io 🗄️

![CI](https://github.com/xuoxod/network_scanner/actions/workflows/discovery.yml/badge.svg) ![docs.rs](https://docs.rs/io/badge.svg) ![crates.io](https://img.shields.io/crates/v/io.svg)

I/O adapters and data loaders. Responsibilities:

- Load and normalize OUI/vendor CSV (`crates/io/data/oui.csv`). This file is included in the repository and is intentionally tracked.
- Provide importers/adapters to ingest legacy netscan outputs and map them into `formats::DiscoveryRecord`.

If you need to update the OUI dataset, replace `crates/io/data/oui.csv` and ensure the format matches the loader expectations (CSV with OUI and vendor fields).

### Build

```bash
# build io as a library (release)
cargo build --manifest-path crates/io/Cargo.toml --lib --release
```

# io

I/O adapters and data loaders. Responsibilities:

- Load and normalize OUI/vendor CSV (`crates/io/data/oui.csv`). This file is included in the repository and is intentionally tracked.
- Provide importers/adapters to ingest legacy netscan outputs and map them into `formats::DiscoveryRecord`.

If you need to update the OUI dataset, replace `crates/io/data/oui.csv` and ensure the format matches the loader expectations (CSV with OUI and vendor fields).
