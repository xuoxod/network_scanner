#!/usr/bin/env bash
# network_scanner scaffold script
# Purpose: initialize a pure Rust project directory in a robust, portable, and opinionated way
# - safe defaults
# - support for binary, lib, or workspace
# - TDD enabled skeleton (tests/, benches/, examples/)
# - optional git init and basic CI template placeholder

set -euo pipefail
IFS=$'\n\t'

SCRIPT_NAME="$(basename "$0")"
ROOT_DIR="$(pwd)"

die() { printf "%s: ERROR: %s\n" "$SCRIPT_NAME" "$*" >&2; exit 1; }

# Defaults
NAME=""
KIND="bin"
WORKSPACE=false
EDITION="2021"
LICENSE="MIT"
GIT=false
CI=false
TESTS=false
AUTHOR=""
DESCRIPTION="A Rust project initialized by scaffold.sh."
FORCE=false
VERBOSE=false
DRY_RUN=false

# ANSI colors
NC="\e[0m"
RED="\e[31m"
GREEN="\e[32m"
YELLOW="\e[33m"
BLUE="\e[34m"
MAGENTA="\e[35m"
CYAN="\e[36m"
BOLD="\e[1m"

# Extra 256-color isotopes for richer output (good on dark backgrounds)
# These use the 256-color foreground escape: \e[38;5;<n>m
ORANGE="\e[38;5;208m"
LIGHT_GREEN="\e[38;5;154m"
LIGHT_CYAN="\e[38;5;87m"
PINK="\e[38;5;205m"
VIOLET="\e[38;5;141m"
TURQUOISE="\e[38;5;44m"
LIME="\e[38;5;118m"
SKY="\e[38;5;117m"
GOLD="\e[38;5;220m"
SALMON="\e[38;5;203m"
LAVENDER="\e[38;5;186m"
OLIVE="\e[38;5;100m"
TEAL="\e[38;5;30m"
CORAL="\e[38;5;209m"
BEIGE="\e[38;5;230m"
INDIGO="\e[38;5;57m"
MAUVE="\e[38;5;170m"

pretty() { printf "%b\n" "${BOLD}${CYAN}==> $*${NC}"; }
info() { printf "%b\n" "${BOLD}${BLUE}[INFO]${NC} $*"; }
# Print debug whenever VERBOSE is on, or during DRY_RUN so dry-runs are very chatty
debug() { if [[ "${VERBOSE}" == true || "${DRY_RUN}" == true ]]; then printf "%b\n" "${MAGENTA}[DEBUG]${NC} $*"; fi }
warn() { printf "%b\n" "${YELLOW}[WARN]${NC} $*"; }
success() { printf "%b\n" "${GREEN}[OK]${NC} $*"; }
err() { printf "%b\n" "${RED}[ERR]${NC} $*"; }

# Additional, more colorful print helpers (useful for dry-run verbose tracing)
note() { printf "%b\n" "${BOLD}${ORANGE}[NOTE]${NC} $*"; }
step() { printf "%b\n" "${BOLD}${SKY}[STEP]${NC} $*"; }
trace() { printf "%b\n" "${LAVENDER}[TRACE]${NC} $*"; }
vdebug() { if [[ "${VERBOSE}" == true || "${DRY_RUN}" == true ]]; then printf "%b\n" "${TEAL}[VDEBUG]${NC} $*"; fi }

# Spinner and dotted inline progress
SPINNER_PID=0
spinner_start() {
  local msg="$1"
  local spinstr='|/-\\'
  printf "%b" "${BOLD}${CYAN}%s ... ${NC}" "$msg"
  (
    i=0
    while :; do
      printf "%c" "${spinstr:i%${#spinstr}:1}"
      sleep 0.12
      printf "\b"
      i=$((i+1))
    done
  ) &
  SPINNER_PID=$!
  debug "spinner pid=$SPINNER_PID"
}

spinner_stop() {
  local msg="${1:-done}"
  if [[ ${SPINNER_PID:-0} -ne 0 ]]; then
    kill "${SPINNER_PID}" >/dev/null 2>&1 || true
    wait "${SPINNER_PID}" 2>/dev/null || true
    SPINNER_PID=0
  fi
  success "$msg"
}

dot_start_pid=0
dot_start() {
  local msg="$1"
  printf "%b" "${BOLD}${CYAN}%s ${NC}" "$msg"
  (
    while :; do
      printf "%b" "."
      sleep 0.6
    done
  ) &
  dot_start_pid=$!
}
dot_stop() {
  local msg="${1:-done}"
  if [[ ${dot_start_pid:-0} -ne 0 ]]; then
    kill "${dot_start_pid}" >/dev/null 2>&1 || true
    wait "${dot_start_pid}" 2>/dev/null || true
    dot_start_pid=0
  fi
  printf "\n"
  success "$msg"
}

# Run commands respecting dry-run and verbose
run_cmd() {
  debug "run_cmd DRY_RUN=${DRY_RUN}: $*"
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would run: $*"
    return 0
  fi
  if [[ "${VERBOSE}" == true ]]; then
    eval "$*"
  else
    eval "$*" >/dev/null 2>&1
  fi
}

run_action() {
  local description="$1"; shift
  pretty "$description"
  if [[ "${DRY_RUN}" == true ]]; then
    # In dry-run, show the precise command array and environment context
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} $description -> $*"
    printf "%b\n" "${MAGENTA}[DRY-RUN-CONTEXT]${NC} PWD=%s, NAME=%s, KIND=%s, WORKSPACE=%s" "$(pwd)" "${NAME}" "${KIND}" "${WORKSPACE}"
    return 0
  fi
  spinner_start "$description"
  trace "Executing command: $*"
  if "$@" >/dev/null 2>&1; then
    spinner_stop "${description} done"
  else
    spinner_stop "${description} failed"
    err "Command failed: $*"
    return 1
  fi
}

backup_if_exists() {
  local path="$1"
  if [[ -e "$path" ]]; then
    local ts
    ts=$(date -u +%Y%m%dT%H%M%SZ)
    local dest="${path}.backup.${ts}"
    if [[ "${DRY_RUN}" == true ]]; then
      printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would move existing ${path} -> ${dest}"
      return 0
    fi
    mv "$path" "$dest"
    success "Moved existing ${path} -> ${dest}"
  fi
}

usage() {
  cat <<USAGE
Usage: $SCRIPT_NAME [options]

Options:
  --name NAME         Project name (defaults to current dir name)
  --bin               Create a binary crate (default)
  --lib               Create a library crate
  --workspace         Create a Cargo workspace and put crates under "crates/"
  --force             Overwrite existing files / crates when present (safe backup)
  --verbose           Enable verbose debug output
  --dry-run           Show what would be done, do not make changes
  --edition ED        Rust edition (2018, 2021). Default: 2021
  --license LICENSE   SPDX license id (MIT, Apache-2.0, etc.) Default: MIT
  --git               Initialize git and create initial commit
  --ci                Add placeholder GitHub Actions workflow files
  --tests             Create test/ integration skeleton and unit-test template
  --author AUTHOR     Author name (optional, for README template)
  --desc DESCRIPTION  Short project description (optional)
  -h, --help          Show this help message

Examples:
  scaffold.sh --name network_scanner --bin --tests --git --ci
  scaffold.sh --lib --name network_core --edition 2021

Flags: --force --verbose --dry-run
USAGE
}

# Argument parsing
while [[ ${#@} -gt 0 ]]; do
  case "$1" in
    --name) NAME="$2"; shift 2;;
    --bin) KIND="bin"; shift;;
    --lib) KIND="lib"; shift;;
    --workspace) WORKSPACE=true; shift;;
    --edition) EDITION="$2"; shift 2;;
    --license) LICENSE="$2"; shift 2;;
    --git) GIT=true; shift;;
    --ci) CI=true; shift;;
    --tests) TESTS=true; shift;;
    --author) AUTHOR="$2"; shift 2;;
    --desc) DESCRIPTION="$2"; shift 2;;
    --force) FORCE=true; shift;;
    --verbose) VERBOSE=true; shift;;
    --dry-run) DRY_RUN=true; shift;;
    -h|--help) usage; exit 0;;
    *) die "Unknown argument: $1";;
  esac
done

if [[ -z "$NAME" ]]; then
  NAME="$(basename "$(pwd)")"
fi

ensure_rust() {
  info "Checking Rust toolchain and cargo..."
  if ! command -v cargo >/dev/null 2>&1; then
    die "cargo not found. Install rustup/cargo first."
  fi
  info "cargo: $(command -v cargo)"
  vdebug "cargo --version: $(cargo --version 2>/dev/null || true)"
  if command -v rustup >/dev/null 2>&1; then
    local t
    t="$(rustup show active-toolchain 2>/dev/null || true)"
    [[ -n "$t" ]] && info "Active toolchain: $t"
  else
    warn "rustup not detected; ensure a Rust toolchain is installed"
  fi
}

create_root_files() {
  info "Preparing top-level files"
  # README
  if [[ ! -f README.md || "${FORCE}" == true ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
      printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would write README.md"
      # show a preview so dry-run users can see the README content that will be written
      printf "%b\n" "${LIGHT_CYAN}[DRY-RUN-PREVIEW]${NC} README.md (first lines):"
      printf "%b\n" "# ${NAME}\n\n${DESCRIPTION}\n\nAuthor: ${AUTHOR}\n"
    else
      cat > README.md <<README
# ${NAME}

${DESCRIPTION}

Author: ${AUTHOR}

How to build:

  cargo build

How to run tests:

  cargo test

README
      success "README.md written"
    fi
  else
    info "README.md exists; use --force to overwrite (will backup)"
  fi

  # .gitignore
  if [[ ! -f .gitignore || "${FORCE}" == true ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
      printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would write .gitignore"
      printf "%b\n" "${LIGHT_CYAN}[DRY-RUN-PREVIEW]${NC} .gitignore (first lines):"
      printf "%b\n" "# Rust/Cargo\n/target\nCargo.lock\n"
    else
      cat > .gitignore <<GITIGNORE
# Rust/Cargo
/target
Cargo.lock

# Editors
*~
.vscode/
.idea/
GITIGNORE
      success ".gitignore written"
    fi
  else
    info ".gitignore exists; use --force to overwrite (will backup)"
  fi

  # LICENSE
  if [[ ! -f LICENSE || "${FORCE}" == true ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
      printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would write LICENSE (${LICENSE})"
      printf "%b\n" "${LIGHT_CYAN}[DRY-RUN-PREVIEW]${NC} LICENSE (first lines):"
      printf "%b\n" "MIT License\n\nCopyright (c) YEAR\n"
    else
      cat > LICENSE <<LICENSE_TXT
MIT License

Copyright (c) YEAR

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
LICENSE_TXT
      success "LICENSE (${LICENSE}) written"
    fi
  else
    info "LICENSE exists; use --force to overwrite (will backup)"
  fi
}

create_workspace() {
  if [[ -f Cargo.toml && $(grep -E '^\[workspace\]' Cargo.toml || true) && "${FORCE}" != true ]]; then
    info "Workspace already present; skipping"
    return
  fi
  info "Creating workspace Cargo.toml"
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would write workspace Cargo.toml and ensure crates/ exists"
    printf "%b\n" "${LIGHT_CYAN}[DRY-RUN-PREVIEW]${NC} Cargo.toml (workspace) snippet:"
    printf '%s\n' "[workspace]" "members = [" "  \"crates/*\"," "]"
  else
    if [[ -f Cargo.toml && "${FORCE}" == true ]]; then
      backup_if_exists Cargo.toml
    fi
    cat > Cargo.toml <<CARGO
[workspace]
members = [
  "crates/*",
]

CARGO
    mkdir -p crates
    success "Workspace Cargo.toml written and crates/ created"
  fi
}

create_crate() {
  local crate_dir="$1"; shift
  local crate_name="${1:-$(basename "$crate_dir")}"; shift || true
  local crate_kind="${1:-$KIND}"

  if [[ -d "$crate_dir" && -f "$crate_dir/Cargo.toml" && "${FORCE}" != true ]]; then
    info "Crate $crate_dir already exists; skipping"
    debug "Found $crate_dir/Cargo.toml, contents (first line): $(head -n 1 "$crate_dir/Cargo.toml" 2>/dev/null || true)"
    return
  fi
  if [[ -d "$crate_dir" && "${FORCE}" == true ]]; then
    backup_if_exists "$crate_dir"
  fi
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would create crate: $crate_kind at $crate_dir (name=$crate_name)"
    printf "%b\n" "${MAGENTA}[DRY-RUN-CMD]${NC} cargo init --vcs none --name $crate_name --edition $EDITION --${crate_kind}"
    return
  fi
  mkdir -p "$crate_dir"
  pushd "$crate_dir" >/dev/null
  if [[ "$crate_kind" == "bin" ]]; then
    cargo init --bin --name "$crate_name" --edition "$EDITION" --vcs none
  else
    cargo init --lib --name "$crate_name" --edition "$EDITION" --vcs none
  fi
  popd >/dev/null
  success "cargo init ($crate_dir) done"
}

add_tests_skeleton() {
  local crate_path="$1"
  # Add a simple unit test to lib.rs or main.rs if present
  if [[ -f "$crate_path/src/lib.rs" && ! $(grep -F "fn it_works" "$crate_path/src/lib.rs" || true) ]]; then
    cat >> "$crate_path/src/lib.rs" <<'RUST'

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
RUST
    info "Added unit test template to $crate_path/src/lib.rs"
  elif [[ -f "$crate_path/src/main.rs" && ! $(grep -F "fn it_works" "$crate_path/src/main.rs" || true) ]]; then
    cat >> "$crate_path/src/main.rs" <<'RUST'

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
RUST
    info "Added unit test template to $crate_path/src/main.rs"
  fi

  mkdir -p "$crate_path/tests"
  if [[ ! -f "$crate_path/tests/integration_test.rs" ]]; then
    cat > "$crate_path/tests/integration_test.rs" <<'RUST'
#[test]
fn integration_example() {
    assert!(true);
}
RUST
    info "Added integration test skeleton to $crate_path/tests"
  fi
}

create_ci_placeholder() {
  if [[ "${CI}" != true ]]; then return; fi
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would create .github/workflows/rust.yml"
    printf "%b\n" "${LIGHT_CYAN}[DRY-RUN-PREVIEW]${NC} .github/workflows/rust.yml (snippet):"
    printf '%s\n' "name: Rust CI" "on: [push, pull_request]" "jobs:" "  build:"
    return
  fi
  mkdir -p .github/workflows
  cat > .github/workflows/rust.yml <<'YAML'
name: Rust CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --workspace --all-targets
      - name: Test
        run: cargo test --workspace --all-targets
YAML
  success "Created .github/workflows/rust.yml"
}

maybe_init_git() {
  if [[ "${GIT}" != true ]]; then return; fi
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}[DRY-RUN]${NC} Would initialize git repo and make initial commit"
    return
  fi
  if [[ -d .git ]]; then
    info "Top-level git repository already exists; skipping git init"
    return
  fi
  git init -b main
  git add .
  git commit -m "chore: initial scaffold for ${NAME}"
  success "Initialized top-level git repository and made initial commit"
}

create_discovery_template() {
  local base="${1:-crates/discovery}"
  info "Creating example discovery crate at ${base}"
  create_crate "$base" discovery lib
  if [[ "${DRY_RUN}" == true ]]; then
    return
  fi
  debug "Writing discovery lib.rs with DiscoveryRecord, trait and SimpleDiscover implementation"
  cat > "$base/src/lib.rs" <<'RUST'
//! Example discovery crate
//! Provides a small `Discover` trait and a SimpleDiscover implementation for tests

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryRecord {
    pub ip: String,
    pub port: Option<u16>,
    pub banner: Option<String>,
}

pub trait Discover {
    fn discover(&self, target: &str) -> Vec<DiscoveryRecord>;
}

pub struct SimpleDiscover;

impl Discover for SimpleDiscover {
    fn discover(&self, target: &str) -> Vec<DiscoveryRecord> {
        // deterministic, test-friendly placeholder
        vec![DiscoveryRecord { ip: target.to_string(), port: Some(80), banner: Some("example".to_string()) }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_discover_returns_record() {
        let d = SimpleDiscover;
        let out = d.discover("192.0.2.1");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].ip, "192.0.2.1");
    }
}
RUST
  add_tests_skeleton "$base"
  success "Example discovery crate created at ${base}"
}

main() {
  pretty "Scaffolding project: ${NAME}"
  if [[ "${DRY_RUN}" == true ]]; then
    printf "%b\n" "${YELLOW}==> DRY-RUN MODE: no files will be changed${NC}"
  fi

  ensure_rust
  # Backup existing top level files if force is requested
  if [[ "${FORCE}" == true && "${DRY_RUN}" != true ]]; then
    info "--force provided: existing files will be moved to backups where needed"
  fi

  # Top-level files
  create_root_files

  # Workspace or single crate
  if [[ "${WORKSPACE}" == true ]]; then
    create_workspace
    # create an example crate in crates/
    create_crate "crates/${NAME}" "${NAME}" "$KIND"
    if [[ "${TESTS}" == true || "${KIND}" == "lib" ]]; then
      add_tests_skeleton "crates/${NAME}"
    fi
    create_discovery_template "crates/discovery"
  else
    # single crate at project root
    create_crate "." "${NAME}" "$KIND"
    if [[ "${TESTS}" == true || "${KIND}" == "lib" ]]; then
      add_tests_skeleton "."
    fi
    create_discovery_template "discovery"
  fi

  create_ci_placeholder
  maybe_init_git

  success "Scaffold complete — ${NAME}"
  info "Next: cd to the project and run 'cargo build' and 'cargo test'"
}

main "$@"
