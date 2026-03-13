#!/usr/bin/env bash
# PolyCredo Editor — Wrapper s omezenim vlaken pro Rust runtime
# CPU: 50% dostupnych jader (via RAYON_NUM_THREADS + TOKIO_WORKER_THREADS)

# Get total CPU cores and calculate 50%
TOTAL_CORES=$(nproc)
LIMIT_CORES=$((TOTAL_CORES / 2))
# Ensure at least 1 core is used
if [ "$LIMIT_CORES" -lt 1 ]; then LIMIT_CORES=1; fi

# Environment variables for Rust libraries (Rayon, Tokio)
export RAYON_NUM_THREADS="$LIMIT_CORES"
export TOKIO_WORKER_THREADS="$LIMIT_CORES"

exec /usr/lib/polycredo-editor/polycredo-editor-bin "$@"
