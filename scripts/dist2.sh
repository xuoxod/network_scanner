#!/usr/bin/env bash
set -euo pipefail

# scripts/dist2.sh
# Build release artifacts for all crates, collect release binaries into
# dist/bin, copy README/LICENSE and data files, and create a tarball.
# Usage: ./scripts/dist2.sh [version]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTDIR="$REPO_ROOT/dist"
BIN_DIR="$OUTDIR/bin"

#!/usr/bin/env bash
set -euo pipefail

# Deprecated shim: dist2.sh kept for backwards compatibility but now
# delegates to scripts/dist.sh which contains the canonical packaging logic.
# This file will be removed in a future cleanup. Use scripts/dist.sh instead.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$SCRIPT_DIR/dist.sh" "$@"
    fi
