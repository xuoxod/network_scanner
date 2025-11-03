#!/usr/bin/env bash
set -euo pipefail

# scripts/dist.sh
# Build release artifacts for all crates, collect release binaries into
# dist/bin, copy README/LICENSE and data files, and create a tarball.
# Usage: ./scripts/dist.sh [version]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTDIR="$REPO_ROOT/dist"
BIN_DIR="$OUTDIR/bin"

if [ "$#" -ge 1 ]; then
  #!/usr/bin/env bash
  set -euo pipefail

  # scripts/dist.sh - clean, single copy
  REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  OUTDIR="$REPO_ROOT/dist"
  BIN_DIR="$OUTDIR/bin"

  #!/usr/bin/env bash
  set -euo pipefail

  # scripts/dist.sh
  # Build release artifacts for all crates, collect release binaries into
  # dist/bin, copy README/LICENSE and data files, and create a tarball.
  # Usage: ./scripts/dist.sh [version]

  REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  OUTDIR="$REPO_ROOT/dist"
  BIN_DIR="$OUTDIR/bin"

  if [ "$#" -ge 1 ]; then
    VERSION="$1"
  else
    # prefer an exact git tag when available, otherwise fall back to latest tag
    # or a short commit id to produce a deterministic tarball name.
    if git -C "$REPO_ROOT" describe --tags --exact-match >/dev/null 2>&1; then
      VERSION="$(git -C "$REPO_ROOT" describe --tags --exact-match)"
    else
      VERSION="$(git -C "$REPO_ROOT" describe --tags --abbrev=0 2>/dev/null || true)"
      if [ -z "$VERSION" ]; then
        VERSION="0.0.0-$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
      fi
    fi
  fi

  echo "Packaging network_scanner version: $VERSION"

  rm -rf "$OUTDIR"
  mkdir -p "$BIN_DIR"

  export CARGO_TARGET_DIR="$OUTDIR/target"

  if [ -f "$REPO_ROOT/Makefile" ]; then
    echo "Running 'make release-all' from repo root (Makefile found)"
    (cd "$REPO_ROOT" && make release-all)
  else
    echo "No top-level Makefile. Building each crate via cargo..."
    for m in "$REPO_ROOT"/crates/*/Cargo.toml; do
      echo "Building $m"
      cargo build --manifest-path "$m" --release
    done
  fi

  echo "Collecting release binaries to $BIN_DIR"
  if [ -d "$CARGO_TARGET_DIR/release" ]; then
    find "$CARGO_TARGET_DIR/release" -maxdepth 1 -type f -executable -print0 \
      | xargs -0 -I{} cp -v {} "$BIN_DIR/" || true
  else
    echo "Warning: release target dir not found: $CARGO_TARGET_DIR/release"
  fi

  # Copy helpful artifacts and docs
  cp -v "$REPO_ROOT/README.md" "$OUTDIR/" || true
  cp -v "$REPO_ROOT/LICENSE" "$OUTDIR/" || true

  # Include the canonical OUI CSV if present
  if [ -f "$REPO_ROOT/crates/io/data/oui.csv" ]; then
    mkdir -p "$OUTDIR/data"
    cp -v "$REPO_ROOT/crates/io/data/oui.csv" "$OUTDIR/data/oui.csv"
  fi

  # Produce a tarball
  TARBALL="$REPO_ROOT/network_scanner-${VERSION}.tar.gz"
  echo "Creating tarball $TARBALL"
  tar -C "$OUTDIR" -czvf "$TARBALL" .

  # Generate SHA256 checksum next to the tarball
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$TARBALL" > "${TARBALL}.sha256"
    echo "Wrote checksum: ${TARBALL}.sha256"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$TARBALL" > "${TARBALL}.sha256"
    echo "Wrote checksum: ${TARBALL}.sha256"
  else
    echo "Warning: no sha256 tool found (sha256sum/shasum); skipping checksum generation"
  fi

  echo "Done. Dist available at: $OUTDIR and tarball: $TARBALL"

  echo "Tip: make the script executable: chmod +x scripts/dist.sh"

