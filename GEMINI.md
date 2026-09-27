# 🦀 SOVEREIGN NETWORK SCANNER ARCHITECTURAL & SECURITY STANDARD

> **Document ID:** `AGY-RULE-SOVEREIGN-NETSCAN-01`  
> **Status:** Permanent Workspace Rulebook  
> **Inherited Standards:** `AGY-RULE-SOVEREIGN-FLAGSHIP-01` & `AGY-RULE-SOVEREIGN-SECURITY-01`  
> **Target System:** `network_scanner` Standalone Engine & Tooling  

---

## 🎯 1. One-Job-Principle (OJP) & Stateless Scout Boundary

* **Stateless Network Scout**:
  * `network_scanner` is strictly a high-performance, stateless CLI scout and embedded library.
  * Zero database bloat: stateful long-term storage, diffing, and forensic investigations belong exclusively in `nexus-recon` and `sovereign-db`.
* **Passive-First Invariant**:
  * All subnet sweeps operate strictly passively by interrogating local kernel ARP tables (`/proc/net/arp` and `ip neigh`).
  * Zero unsolicited network packets are transmitted unless active probing (`--probe`) or port scanning (`--portscan`) is explicitly toggled by the operator.
* **Deterministic Canonical Contracts**:
  * All serialization must strictly satisfy [`CONTRACTS.md`](CONTRACTS.md) (uniform 6-column CSV, Target JSON, Legacy JSON).

---

## ⚡ 2. Pure-Rust Systems Engineering & Static Distribution

* **Zero-Dependency Static Musl Linking**:
  * Targets `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` with 100% static linking (`make musl`).
  * Produces single, self-contained binaries (`discovery-cli`, `netcheck`) executable across any Linux kernel without glibc dependencies.
* **Panic Immunity & Error Discipline**:
  * Production paths must never call `unwrap()` or `expect()`.
  * Return descriptive typed domain errors or graceful recovery fallbacks.
* **Bounded Resource Allocation**:
  * Thread pools and concurrency must be bounded (`--concurrency`, default: 64).
  * Socket operations must enforce explicit timeouts (`--timeout`, default: 1s).
  * CIDR expansions must reject ranges wider than `/16` to prevent multi-gigabyte memory exhaustion.

---

## 🗄️ 3. Embedded IEEE OUI Resolution & Dynamic Caching

* **Offline-First Zero-Latency Resolution**:
  * Includes an embedded in-tree database of 32,800+ authoritative IEEE OUI assignments (`crates/io/data/oui.csv`).
  * Resolves MAC addresses to vendor names entirely in-memory in $< 1\mu\text{s}$ without external API queries or telemetry leakage.
* **Deterministic 3-Tier Cache Precedence**:
  1. `NETWORK_SCANNER_OUI_PATH` environment variable override (air-gapped environments)
  2. Local user XDG cache: `~/.local/share/network_scanner/oui.csv` (populated via `--update-oui`)
  3. Embedded static IEEE dataset fallback shipped inside the binary

---

## 🛡️ 4. The 6-Tier Sovereign Verification & Adversarial Shield

Every crate and parser boundary must satisfy the 6 testing tiers:

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    THE 6-TIER SOVEREIGN VERIFICATION MATRIX                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│ Tier 1: POC TDD        │ Algorithmic correctness verified test-first before code│
│ Tier 2: RW (Real-World)│ Production ARP tables, actual router banners & devices │
│ Tier 3: EC (Edge-Case) │ Single-host /32, RFC 3021 /31 point-to-point subnets   │
│ Tier 4: SIM (Chaos)    │ Packet drops, simulated jitter, sudden socket closure  │
│ Tier 5: TDD CLI Tools  │ Standalone test harness and golden file verification   │
│ Tier 6: RED-TEAM ATTACK│ Deliberate self-fuzzing & exploitation test harness    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

* **Adversarial Self-Attack Invariants (`tests/adversarial_*_tests.rs`)**:
  * **CSV Formula Injection**: All exported strings starting with `=`, `+`, `-`, `@`, `\t`, or `\r` must be sanitized with single-quote escaping (`'`) before CSV writing to prevent spreadsheet execution.
  * **Terminal ANSI Escape Scrubbing**: All banners scraped from untrusted network ports must be stripped of ANSI CSI sequences before console output or logging.
  * **CIDR DoS / OOM Guardrails**: Any CIDR prefix $< /16$ must be rejected immediately to safeguard host memory.
  * **Kernel ARP Table Validation**: Hardware addresses from `/proc/net/arp` and `ip neigh` must pass hex validation (`parse_mac`) prior to record ingestion.

---

## 🔒 5. Zero-Leak Sanitization Directive

* **Strict Public Hygiene**:
  * Under no circumstances may private usernames, internal IP topologies, or private hostnames be committed to public repositories.
  * Automated diff audits (`git diff --cached`) must be executed prior to every commit.
