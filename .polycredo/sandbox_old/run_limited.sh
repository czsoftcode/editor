#!/usr/bin/env bash
# PolyCredo Editor — Lokální spouštěcí skript s dynamickým omezením prostředků
# CPU: 50% dostupných jader
# RAM: 66% celkové fyzické paměti

# Zjištění počtu jader a výpočet 50%
TOTAL_CORES=$(nproc)
LIMIT_CORES=$((TOTAL_CORES / 2))
if [ "$LIMIT_CORES" -lt 1 ]; then LIMIT_CORES=1; fi
CPU_QUOTA=$((LIMIT_CORES * 100))

# Zjištění celkové RAM a výpočet 33% (1/3)
TOTAL_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
LIMIT_KB=$((TOTAL_KB * 2 / 3))
LIMIT_MB=$((LIMIT_KB / 1024))

echo "==> Spouštím PolyCredo Editor s dynamickými limity:"
echo "    CPU: $LIMIT_CORES jader ($CPU_QUOTA%)"
echo "    RAM: $LIMIT_MB MB (66% z celkové kapacity)"
echo "    Příznak: --new-instance"

# Nastavení proměnných pro Rust knihovny (Rayon, Tokio)
export RAYON_NUM_THREADS="$LIMIT_CORES"
export TOKIO_WORKER_THREADS="$LIMIT_CORES"

# Spuštění přes systemd-run (pokud je k dispozici)
if command -v systemd-run >/dev/null 2>&1; then
    exec systemd-run --user --scope \
        --description="PolyCredo Editor Dev (Limited Resources)" \
        -p CPUQuota="${CPU_QUOTA}%" \
        -p MemoryMax="${LIMIT_MB}M" \
        cargo run -- --new-instance "$@"
else
    echo "Varování: systemd-run nebyl nalezen, limity RAM/CPU na úrovni OS nebudou vynuceny."
    exec cargo run -- --new-instance "$@"
fi
