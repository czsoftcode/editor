#!/usr/bin/env bash
# Centralized versioning script for PolyCredo Editor
# Usage: ./scripts/get-version.sh [deb|deb-dev|arch|generic]

set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 1. Get base version from Cargo.toml (e.g., 0.8.0-dev)
BASE_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"

TYPE="${1:-generic}"

case "$TYPE" in
    deb-dev)
        # Development Debian version with build number: 0.8.0-dev-build223
        BUILD_FILE="$ROOT_DIR/.build_number"
        BUILD_NUMBER="$(tr -d '[:space:]' < "${BUILD_FILE:-/dev/null}" 2>/dev/null || echo "0")"
        echo "${BASE_VERSION}-build${BUILD_NUMBER}"
        ;;
    deb)
        # Standard Debian version for releases: 0.8.0-dev
        echo "${BASE_VERSION}"
        ;;
    arch|aur)
        # Arch Linux format: 0.8.0.dev.build223
        BUILD_FILE="$ROOT_DIR/.build_number"
        BUILD_NUMBER="$(tr -d '[:space:]' < "${BUILD_FILE:-/dev/null}" 2>/dev/null || echo "0")"
        echo "${BASE_VERSION//-/.}.build${BUILD_NUMBER}"
        ;;
    *)
        # Result: 0.8.0-dev (Cargo, Snap, etc.)
        echo "${BASE_VERSION}"
        ;;
esac
