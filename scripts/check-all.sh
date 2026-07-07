#!/usr/bin/env bash
# Local pre-push gate - the same checks CI runs. Run from the repo root.
#
# Core Rust checks always run (and must pass). Supply-chain tools run if
# installed; if a tool is missing locally it is skipped with a note (CI installs
# and runs all of them, so nothing is silently dropped there).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

run() { printf '\n\033[1m== %s ==\033[0m\n' "$*"; "$@"; }

# --- Core (required) ---
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace

# --- Supply chain (run if available) ---
optional() {
    local tool="$1"; shift
    if command -v "$tool" >/dev/null 2>&1; then
        run "$@"
    else
        printf '\n\033[33m-- skipping %s (not installed) --\033[0m\n' "$tool"
    fi
}

optional cargo-audit   cargo audit
optional cargo-deny    cargo deny check
optional gitleaks      gitleaks detect --no-banner --redact --config .gitleaks.toml
optional typos         typos

printf '\n\033[32mAll checks passed.\033[0m\n'
