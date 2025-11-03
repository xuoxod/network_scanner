# BUILDING network_scanner

Practical build, test and packaging examples for the network_scanner
workspace. Each crate lives under `crates/<name>/` and has its own
`Cargo.toml`.

Location: repository root: `BUILDING.md`

## Prerequisites

- Rust toolchain (rustup + cargo). Recommended: stable >= 1.70.
- Standard build tools (gcc, make). For static musl builds you may need
  additional system packages (musl-dev, musl-tools).

> Building does not require elevated privileges. Running active network
> probes may require sudo/root — use `sudo -E` only when you intend to run
> privileged probes, not for building.

## Quick conventions

- From repo root: use `--manifest-path crates/<crate>/Cargo.toml` to build a
  specific crate without changing directories.
- Or `cd crates/<crate>` and run `cargo <cmd>` locally in the crate folder.

## Build examples

Build the `discovery` CLI (debug):

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli
```

Build the `discovery` CLI (release):

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli --release
```

Build `netutils` as a library (release):

```bash
cargo build --manifest-path crates/netutils/Cargo.toml --lib --release
```

If you prefer to work inside the crate:

```bash
cd crates/discovery
cargo build --release
```

## Centralized output (useful for CI / packaging)

Set `CARGO_TARGET_DIR` to place all artifacts in one location (e.g. `dist`):

```bash
mkdir -p dist
CARGO_TARGET_DIR=dist/target cargo build --manifest-path crates/discovery/Cargo.toml --release
# binary available at dist/target/release/discovery-cli
```

## Build all crates (quick)

Use the repository `Makefile` targets (recommended):

```bash
make build-all    # or
make release-all
```

Or run a short loop:

```bash
for m in crates/*/Cargo.toml; do cargo build --manifest-path "$m" --release || break; done
```

## Running binaries

Run the release binary directly:

```bash
./crates/discovery/target/release/discovery-cli 10.0.0.0/24 --out out.csv
```

Or use `cargo run` (convenient during development):

```bash
cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --out run.csv
```

## Tests

Run all tests for `discovery`:

```bash
cargo test --manifest-path crates/discovery/Cargo.toml
```

Run a single test by name:

```bash
cargo test --manifest-path crates/io/Cargo.toml exported_json_has_expected_shape
```

## Formatting and linting

```bash
cargo fmt --manifest-path crates/discovery/Cargo.toml
cargo clippy --manifest-path crates/discovery/Cargo.toml -- -D warnings
```

## Docs

```bash
cargo doc --manifest-path crates/formats/Cargo.toml --no-deps --open
```

## Packaging and distribution (example)

The included script `scripts/dist.sh` builds release artifacts to a
centralized `dist/target`, copies selected files into `dist/` and creates a
versioned tarball `network_scanner-<version>.tar.gz`.

From the repo root:

```bash
chmod +x scripts/dist.sh
./scripts/dist.sh v0.2.0
```

The script also produces a SHA256 file next to the tarball (e.g.
`network_scanner-v0.2.0.tar.gz.sha256`).

## Verifying downloads (SHA256 and optional GPG)

After downloading a release tarball, verify the checksum before extracting:

Linux / macOS:

```bash
sha256sum -c network_scanner-v0.2.0.tar.gz.sha256
# or
sha256sum network_scanner-v0.2.0.tar.gz
cat network_scanner-v0.2.0.tar.gz.sha256
```

Windows (PowerShell):

```powershell
CertUtil -hashfile network_scanner-v0.2.0.tar.gz SHA256
```

Optional GPG verification (requires maintainer public key):

```bash
gpg --import maintainer_pubkey.asc
gpg --verify network_scanner-v0.2.0.tar.gz.sig network_scanner-v0.2.0.tar.gz
```

## CI notes

- The repository includes a GitHub Actions workflow `.github/workflows/release.yml`
   that runs on tag pushes, builds/tests, runs `scripts/dist.sh`, and
  uploads release artifacts with a SHA256 file.
- Consider adding caching for `~/.cargo/registry` and `CARGO_TARGET_DIR` in CI to speed builds.

## Reproducible data

The project includes `crates/io/data/oui.csv` used for vendor lookups. Keep
it in the tree to ensure tests and exports are reproducible.

---

If you'd like, I can add: SHA256 verification examples in the README,
GPG signing in CI (requires a key/secret), or a Windows PowerShell
`dist.ps1` equivalent. Tell me which and I'll add it next.

# BUILDING network_scanner

This file collects practical, copy-pasteable build examples for the
`network_scanner` workspace. The repository contains multiple independent
crates (no single top-level workspace manifest). Each crate has its own
`Cargo.toml` under `crates/<name>/Cargo.toml`.

Location: `BUILDING.md` (repo root)

## Prerequisites

- Rust toolchain (rustup + cargo). Recommended stable >= 1.70.
- Basic build tools (gcc/musl toolchain if you plan static builds).
- Elevated privileges (sudo) only when running active network probes that
  require raw sockets — building does not need sudo.

## Where to run commands

- From the repository root: use `--manifest-path crates/<crate>/Cargo.toml`
  to build a single crate without changing directories.
- Or change directory into a crate and run `cargo <cmd>` from there. Both
  approaches are shown below.

## Per-crate build examples (debug / release)

Build the `discovery` CLI in debug (fast, default):

```bash
# from repo root
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli
```

Build the `discovery` CLI in release (optimized):

```bash
cargo build --manifest-path crates/discovery/Cargo.toml --bin discovery-cli --release
```

Build the `netutils` crate as a library in release mode:

```bash
cargo build --manifest-path crates/netutils/Cargo.toml --lib --release
```

If you prefer to `cd` into the crate:

```bash
cd crates/discovery
cargo build --bin discovery-cli --release
```

## Where build artifacts appear

By default artifacts are written under the crate's `target/` folder. For
`crates/discovery` the release binary lives at
`crates/discovery/target/release/discovery-cli` and debug at
`crates/discovery/target/debug/discovery-cli`.

If you want a single, centralized output directory for multiple crates (CI
or packaging), set `CARGO_TARGET_DIR`:

```bash
# centralize all outputs under ./dist/target
mkdir -p dist
CARGO_TARGET_DIR=dist/target cargo build --manifest-path crates/discovery/Cargo.toml --release

# artifact available at dist/target/release/discovery-cli
```

## Build all crates quickly

Use the included `Makefile` from the repo root:

```bash
make build-all    # builds debug or release depending on Makefile target
make release-all  # builds release for all crates
```

Or a plain shell loop (portable):

```bash
for m in crates/*/Cargo.toml; do
  cargo build --manifest-path "$m" --release || break
done
```

## Running binaries after building

Run the binary directly from its target directory:

```bash
./crates/discovery/target/release/discovery-cli 10.0.0.0/24 --out out.csv
```

Or use `cargo run` for a quick run (compiles if needed):

```bash
cargo run --manifest-path crates/discovery/Cargo.toml -- 10.0.0.0/24 --out run.csv
```

Notes about permissions: ARP/raw-socket probes may require elevated
privileges. Use `sudo -E` when you intend to run active probes (not for
building).

## Using the produced library in another Rust project

If you want to use `netutils` (or another crate) from a different local
project, add a path dependency in the consuming project's `Cargo.toml`:

```toml
[dependencies]
netutils = { path = "/absolute/path/to/network_scanner/crates/netutils" }
```

Then `cargo build` in the consumer project will build the local `netutils`.

## Running tests and specific test examples

Run all tests for `discovery`:

```bash
cargo test --manifest-path crates/discovery/Cargo.toml
```

Run a single test by name (useful during development):

```bash
cargo test --manifest-path crates/io/Cargo.toml exported_json_has_expected_shape
```

Run release-mode tests (rare):

```bash
RUSTFLAGS='-C opt-level=3' cargo test --manifest-path crates/netutils/Cargo.toml --release
```

## Formatting and linting

Use `rustfmt` and `clippy` per crate:

```bash
cargo fmt --manifest-path crates/discovery/Cargo.toml
cargo clippy --manifest-path crates/discovery/Cargo.toml -- -D warnings
```

## Generating documentation

Build docs for `formats` and open locally:

```bash
cargo doc --manifest-path crates/formats/Cargo.toml --no-deps --open
```

Aggregate docs for multiple crates (using centralized target dir):

```bash
CARGO_TARGET_DIR=dist/target cargo doc --manifest-path crates/formats/Cargo.toml --no-deps
# docs at dist/target/doc
```

## Packaging binaries for release (example)

Create a redistributable `dist/` that contains release binaries and a
README. Example packaging steps:

```bash
rm -rf dist && mkdir -p dist/bin

# build release artifacts to a centralized location
CARGO_TARGET_DIR=dist/target make release-all

# copy the discovery CLI to dist/bin
cp dist/target/release/discovery-cli dist/bin/

# create a tarball
tar -C dist -czvf network_scanner-0.1.0.tar.gz .
```

## Cross-compilation hints (musl static binary)

To produce a static Linux binary (musl):

```bash
rustup target add x86_64-unknown-linux-musl
CARGO_TARGET_DIR=dist/target cargo build --manifest-path crates/discovery/Cargo.toml --release --target x86_64-unknown-linux-musl

# You may need to install musl-toolchain (e.g., musl-dev) and linker helpers on your distro.
```

## CI-friendly recommendations

- Set `CARGO_TARGET_DIR` in CI to cache artifacts across jobs.
- Run `cargo clippy` with `-D warnings` to keep CI strict.
- Use the `Makefile` targets for consistent builds between local dev and CI.

## Reproducible data note

The repository includes a canonical OUI vendor CSV used by the IO crate at
`crates/io/data/oui.csv`. Keep that file in the tree for reproducible
vendor lookups in tests and releases.

## Verifying downloads (SHA256 and optional GPG)

When you download a release tarball from GitHub, verify the checksum to
ensure the file wasn't corrupted or tampered with. The release pipeline
attaches a `.sha256` file next to the tarball.

On Linux/macOS:

```bash
# verify using the supplied sha256 file (recommended)
sha256sum -c network_scanner-0.2.0.tar.gz.sha256

# or inspect the hash manually
sha256sum network_scanner-0.2.0.tar.gz
# then compare the printed hash with the value in the .sha256 file
cat network_scanner-0.2.0.tar.gz.sha256
```

On Windows (PowerShell):

```powershell
# compute a hash for manual compare
CertUtil -hashfile network_scanner-0.2.0.tar.gz SHA256
```

Optional GPG verification

If the release also includes a detached GPG signature (e.g. `.sig`) you
can verify the archive was signed by a trusted key. The repository does
not publish a signing key automatically; if/when a maintainer publishes a
public key, import it first:

```bash
# import the maintainer's public key file (provided separately)
gpg --import maintainer_pubkey.asc

# verify the signature
gpg --verify network_scanner-0.2.0.tar.gz.sig network_scanner-0.2.0.tar.gz
```

If you want, I can extend the CI workflow and `scripts/dist.sh` to produce
an optional GPG signature when a signing key is provided via CI secrets.

## Where this file lives

This document: `BUILDING.md` at the repository root. If you want more
examples (packaging for a specific platform, or a CI YAML snippet) tell me
which target system and I will add it.

---

# BUILDING network_scanner

*** End Patch
This file collects practical, copy-pasteable build examples for the
