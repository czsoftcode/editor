---
phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
plan: 02
subsystem: ui
tags: [egui, terminal, sandbox, tabs]

# Dependency graph
requires:
  - phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
    provides: runtime apply sandbox režimu z plánu 05-01
provides:
  - Graceful restart terminálů při runtime změně sandbox režimu
  - Přemapování otevřených tabů mezi rooty s označením chybějících souborů
  - Prompt pro remap tabů po přepnutí režimu
affects: [workspace, terminal, editor, file-tree]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Retired terminal sessions pro doběhnutí běžících procesů", "Tab remap summary s reloadem jen u nemodifikovaných tabů"]

key-files:
  created: []
  modified:
    - src/app/ui/terminal/mod.rs
    - src/app/ui/terminal/instance/mod.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/editor/files.rs
    - src/app/ui/panels.rs
    - src/app/types.rs
    - locales/cs/ui.ftl
    - locales/en/ui.ftl

key-decisions:
  - "Label režimu terminálu odvozovat z reálného working dir, aby se změna ukázala až po restartu."
  - "Při přepnutí sandbox režimu nabídnout remap otevřených tabů a chybějící soubory ponechat otevřené s varováním."

patterns-established:
  - "Retire + tick pro terminály: exit je poslán, instance zůstává, dokud nepřijde Exit event."
  - "Remap tabů podle relativních cest s reloadem jen u nemodifikovaných souborů."

requirements-completed: [SANDBOX-03]

# Metrics
duration: 17m
completed: 2026-03-05
---

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
