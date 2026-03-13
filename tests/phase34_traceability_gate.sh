#!/usr/bin/env bash
set -euo pipefail

rg -n "R33-A|R33-B|R33-C|R33-D" \
  .planning/REQUIREMENTS.md \
  .planning/ROADMAP.md \
  .planning/STATE.md \
  .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md -S >/dev/null

! rg -n "status:\\s*gaps_found" \
  .planning/phases/33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu/33-VERIFICATION.md \
  .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md -S >/dev/null

# Explicitni closure marker pridany v GREEN kroku.
rg -n "^final_gate:\\s*passed" .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md -S >/dev/null

echo "phase34 traceability gate: PASS"
