#!/usr/bin/env bash
# PolyCredo Editor — Dynamic Resource Limiter Wrapper
# Calculates limits based on current system resources:
# - CPU: 50% of available cores
# - RAM: 66% of total physical memory

# Get total CPU cores and calculate 50%
TOTAL_CORES=$(nproc)
LIMIT_CORES=$((TOTAL_CORES / 2))
# Ensure at least 1 core is used
if [ "$LIMIT_CORES" -lt 1 ]; then LIMIT_CORES=1; fi
CPU_QUOTA=$((LIMIT_CORES * 100))

# Get total RAM in kB and calculate 66% (2/3)
TOTAL_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
LIMIT_KB=$((TOTAL_KB * 2 / 3))
LIMIT_MB=$((LIMIT_KB / 1024))

# Environment variables for Rust libraries (Rayon, Tokio)
export RAYON_NUM_THREADS="$LIMIT_CORES"
export TOKIO_WORKER_THREADS="$LIMIT_CORES"

# Run the actual binary via systemd-run for OS-level enforcement
# If systemd-run is not available or fails (e.g. no user session bus), fall back to direct execution
if command -v systemd-run >/dev/null 2>&1; then
    # We don't use 'exec' here immediately so we can catch failure and fall back
    if systemd-run --user --scope \
        --description="PolyCredo Editor (Limited Resources)" \
        -p CPUQuota="${CPU_QUOTA}%" \
        -p MemoryMax="${LIMIT_MB}M" \
        /usr/lib/polycredo-editor/polycredo-editor-bin "$@" 2>/dev/null; then
        exit 0
    fi
fi

# Fallback to direct execution if systemd-run fails or is missing
exec /usr/lib/polycredo-editor/polycredo-editor-bin "$@"
