# network_scanner — I/O contract (v1)

This document defines the canonical input/output contracts for discovery data used across the workspace.

Goal

- Provide a small, stable, testable canonical type for discovery observations.
- Define deterministic mapping rules from legacy netscan JSON/CSV into the canonical form.
- Specify normalization, error modes, and test requirements (golden files, integration tests).

Versioning

- Contract version: v1
- If fields are added later, add them as optional and bump contract version to v2.

Canonical type (DiscoveryRecord v1)

- ip: string — canonical textual IP (IPv4 or IPv6); required.
- port: Option<u16> — observed service port if available; optional.
- banner: Option<string> — free-form banner, hostname, or primary identifying string; optional.

Rationale for minimal fields

- Start with a minimal, stable surface area for unit tests and golden comparisons.
- Additional metadata (MAC, vendor, timestamp, tags) should be captured in separate types or added as optional fields later.

Mapping rules (netscan JSON -> DiscoveryRecord)

- IP: read `IP` (case-insensitive), or `ip`.
- port: if `ports` exists and is a non-empty array, use the first numeric entry as port (truncate to u16). If `ports` absent, leave `port` None.
- banner: prefer `Hostname` (non-empty string). If missing, prefer first element of `banners` array. If both absent, leave None.
- Ignore or skip entries that have no IP or malformed IP — tests should assert how many were skipped.

Mapping rules (netscan CSV -> DiscoveryRecord)

- CSV is expected to follow header: Timestamp,IP,MAC,Hostname,Vendor,OS (but mapping is resilient to different column order).
- IP: map from IP column (best-effort by header name or column index 1 if unknown).
- banner: map from Hostname column if non-empty.
- port: CSV typically contains no port information; set to None.

Normalization rules

- Trim whitespace from all string fields.
- Lowercase hostnames when used for deduping; preserve original casing in `banner` when stored.
- For IP, preserve textual form but validate format; reject invalid addresses in mapping (count and report as skipped).

Dedupe and canonicalization

- Downstream systems should deduplicate by (ip, port) where port None is treated as "host-only" record.
- For golden tests use a stable sorting: sort by ip string, then by port (None last) before serializing.

Error handling and test expectations

- Mapping functions must return a Result<Vec<DiscoveryRecord>, Error>. Failures are I/O or parse errors.
- Individual malformed rows/entries should be skipped with a logged counter; mapping functions may optionally return `(Vec<DiscoveryRecord>, usize_skipped)` in future.

Golden-file tests

- Golden JSON files contain a sorted array of `DiscoveryRecord` values serialized via `serde_json::to_string_pretty` after normalization and sorting.
- Each golden test must document the source file used to generate the golden and the generator command used.

CI and quality gates

- CI must run: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` (optional until code stabilized), and `cargo test --workspace`.
- Golden tests must be executed in CI. If golden files are large, consider storing them under `tests/golden/` and keeping them small (subset) for CI; full datasets can be in a separate data store.

Extending the contract

- To add MAC, vendor, timestamp, or boolean tags, add optional fields to `DiscoveryRecord` and update the generator and tests to include or ignore them accordingly.

Acceptance criteria

- The `formats::DiscoveryRecord` must be able to round-trip JSON and CSV example records for the provided samples.
- Golden tests must pass locally and in CI.
- New fields must not break existing golden tests (add as optional and update goldens deliberately).

Next steps

1. Expand `DiscoveryRecord` if you want more canonical fields (MAC, vendor, timestamp). Confirm which fields to add.
2. Add CI rules (fmt/clippy/tests) and pre-commit hooks.
3. Implement discovery core crates that return `DiscoveryRecord` and write integration tests using replayed inputs.

Contact

- If you want a stricter contract (more fields or stronger validation), tell me which fields to lock down and I'll update this file and the types accordingly.
