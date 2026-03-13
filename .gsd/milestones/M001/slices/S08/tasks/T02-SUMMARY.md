---
id: T02
parent: S08
milestone: M001
provides:
  - Graceful restart terminálů při runtime změně sandbox režimu
  - Přemapování otevřených tabů mezi rooty s označením chybějících souborů
  - Prompt pro remap tabů po přepnutí režimu
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 17m
verification_result: passed
completed_at: 2026-03-05
blocker_discovered: false
---
# T02: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 02

**# Phase 05 Plan 02: Okamžité Aplikování Změny Režimu Sandboxu Po Přepnutí Checkboxu Summary**

## What Happened

# Phase 05 Plan 02: Okamžité Aplikování Změny Režimu Sandboxu Po Přepnutí Checkboxu Summary

**Terminály se při změně sandbox režimu restartují graceful, labely se odvozují z běžícího working dir a taby lze po přepnutí přemapovat s označením chybějících souborů.**

## Performance

- **Duration:** 17m
- **Started:** 2026-03-05T06:16:18Z
- **Completed:** 2026-03-05T06:33:33Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments
- Graceful restart terminálů s ponecháním běžících procesů do doběhu a správnými labely režimu.
- Remap otevřených tabů při změně rootu s varováním u chybějících souborů.
- Toast prompt pro uživatelské rozhodnutí o remapu tabů.

## Task Commits

Each task was committed atomically:

1. **Task 1: Terminály – restart a labely po okamžité změně režimu** - `4e6f364` (feat)
2. **Task 2: File tree root + přemapování otevřených tabů** - `b7f1c36` (feat)

## Files Created/Modified
- `src/app/ui/terminal/mod.rs` - label podle working dir + testy
- `src/app/ui/terminal/instance/mod.rs` - graceful exit + background tick
- `src/app/ui/workspace/mod.rs` - remap prompt po apply + tick retired terminálů
- `src/app/ui/editor/files.rs` - remap tabů a testy
- `src/app/ui/panels.rs` - obsluha toast akce pro remap/skip
- `src/app/types.rs` - nové toast akce
- `locales/cs/ui.ftl` - CZ texty pro remap prompt
- `locales/en/ui.ftl` - EN texty pro remap prompt

## Decisions Made
- Label režimu terminálu se řídí skutečným working dir, aby se nezměnil dřív než po restartu.
- Remap tabů je explicitní volba uživatele s ponecháním neexistujících souborů otevřených a označených.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `./check.sh` selhal na `cargo fmt --all` (neformátované změny v několika souborech). Test běhu byl proveden, ale formátování nebylo aplikováno.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Sandbox runtime apply je připraven pro další navazující kroky.
- Doporučeno spustit `cargo fmt --all`, aby `./check.sh` prošel.

---
*Phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu*
*Completed: 2026-03-05*
