#!/usr/bin/env bash
set -euo pipefail

missing=0

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit not found. Install with: cargo install cargo-audit --locked" >&2
  missing=1
fi

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny not found. Install with: cargo install cargo-deny --locked" >&2
  missing=1
fi

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

cargo check --all-targets
cargo clippy --all-targets
cargo audit --ignore RUSTSEC-2023-0071
cargo deny check
