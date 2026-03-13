#!/usr/bin/env bash
# PolyCredo Editor — Lokalni spousteci skript s omezenim vlaken
# CPU: 50% dostupnych jader (via RAYON_NUM_THREADS + TOKIO_WORKER_THREADS)

# Zjisteni poctu jader a vypocet 50%
TOTAL_CORES=$(nproc)
LIMIT_CORES=$((TOTAL_CORES / 2))
if [ "$LIMIT_CORES" -lt 1 ]; then LIMIT_CORES=1; fi

echo "==> Spoustim PolyCredo Editor s omezenim vlaken:"
echo "    CPU: $LIMIT_CORES vlaken (50% z $TOTAL_CORES jader)"
echo "    Priznak: --new-instance"

# Nastaveni promennych pro Rust knihovny (Rayon, Tokio)
export RAYON_NUM_THREADS="$LIMIT_CORES"
export TOKIO_WORKER_THREADS="$LIMIT_CORES"

exec cargo run -- --new-instance "$@"
