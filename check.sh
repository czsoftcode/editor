#!/bin/bash
# PolyCredo Editor — Quality Gate Check
# Runs all checks required to ensure code quality.

set -e # Exit immediately if a command exits with a non-zero status.

echo "1. Code Formatting (cargo fmt)..."
cargo fmt --all -- --check || (echo "ERROR: Code is not formatted. Run 'cargo fmt --all'." && exit 1)

echo "2. Static Analysis (cargo clippy)..."
cargo clippy --all-targets --all-features -- -D warnings

echo "3. Running Tests (cargo test)..."
cargo test --all-targets --all-features

echo ""
echo "✅ Quality Gate: All checks passed successfully!"
