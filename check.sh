#!/bin/bash
# PolyCredo Editor — Quality Gate Check
# Spustí všechny kontroly potřebné pro zajištění kvality kódu.

set -e # Ukončí skript při jakékoli chybě

echo "1. Formátování kódu (cargo fmt)..."
cargo fmt --all -- --check || (echo "CHYBA: Kód není naformátován. Spusťte 'cargo fmt --all'." && exit 1)

echo "2. Statická analýza (cargo clippy)..."
cargo clippy --all-targets --all-features -- -D warnings

echo "3. Spouštění testů (cargo test)..."
cargo test --all-targets --all-features

echo ""
echo "✅ Quality Gate: Všechny kontroly proběhly úspěšně!"
