#!/usr/bin/env bash
# faf-rust local ship bar — same gates as .github/workflows/ci.yml
# Usage: bash scripts/ci.sh
# Install pre-push: bash scripts/install-hooks.sh
set -euo pipefail

export PATH="${HOME}/.cargo/bin:${PATH:-}"
cd "$(dirname "$0")/.."

echo "==> PATH cargo: $(command -v cargo)"
echo "==> rustc: $(rustc --version)"

echo "==> fmt"
cargo fmt --all -- --check

echo "==> clippy (workspace)"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> test (workspace)"
cargo test --workspace

echo "==> release build (workspace)"
cargo build --workspace --release

echo "✅ scripts/ci.sh green (matches CI)"
