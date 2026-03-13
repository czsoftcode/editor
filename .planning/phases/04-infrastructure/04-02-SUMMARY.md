---
phase: 04-infrastructure
plan: 02
subsystem: ui
tags: [sandbox, settings, tooltip, ux, egui, rust]

requires:
  - phase: 04-infrastructure-01
    provides: Sandbox toggle a texty v Settings
provides:
  - Snadneji objevitelny tooltip u sandbox prepinace
  - Citelnejsi inline poznamka o reopen bez vizualniho potlaceni
affects: [settings, ux, lokalizace]

tech-stack:
  added: []
  patterns:
    - "Tooltip navazany na full-width row response misto male ikony"

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs/settings.rs

key-decisions:
  - "Hover target tooltipu je navazany na cely radek sandbox prepinace."
  - "Poznamka o restartu terminalu po reopen se renderuje bez small() potlaceni."

patterns-established:
  - "U kritickych settings hintu preferovat bezny/strong text pred small+weak kombinaci."

requirements-completed: [SETT-01, SETT-02, SETT-03, SETT-04, SETT-05]

duration: 1min
completed: 2026-03-05
---

# Phase 04 Plan 02: Infrastructure Summary

**Sandbox tooltip je navazany na sirsi hover oblast celeho radku a inline reopen poznamka je citelna bez zbytecneho vizualniho potlaceni.**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-05T04:14:09Z
- **Completed:** 2026-03-05T04:15:50Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Rozsireni hover oblasti tooltipu na cely radek sandbox prepinace.
- Zvyseni vizualni priority informacni ikony a terminal-note textu.
- Oveření, ze lokalizacni texty zustaly beze zmeny.

## Task Commits

Each task was committed atomically:

1. **Task 1: Zviditelneni tooltipu a inline poznamky** - `e393ebd` (fix)
2. **Task 2: Lokalizace (pokud se zmeni texty)** - `bd28751` (chore)

**Plan metadata:** pending

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs/settings.rs` - full-width tooltip hover response + citelnejsi inline poznamka.

## Decisions Made
- Tooltip zustava obsahove stejny, meni se jen discoverability a vizualni priorita.
- Lokalizacni soubory se nemenily, protoze nedoslo ke zmene textoveho obsahu.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `cargo check` blokovany `sccache` v sandboxu**
- **Found during:** Task 1 verification
- **Issue:** `sccache: Operation not permitted (os error 1)` pri `cargo check`.
- **Fix:** Verifikace spustena s `RUSTC_WRAPPER=` (prime volani `rustc` bez sccache wrapperu).
- **Files modified:** none
- **Verification:** `RUSTC_WRAPPER= cargo check` prosel.
- **Committed in:** n/a (runtime verification fix)

---

**Total deviations:** 1 auto-fixed (1x Rule 3 - Blocking)
**Impact on plan:** Bez scope creep; pouze unblock verifikace v omezenem sandbox prostredi.

## Issues Encountered
- `./check.sh` selhava na predexistujicich clippy warning-as-error v nesouvisejicich souborech (`markdown.rs`, `normal.rs`, `terminal/instance/theme.rs`), mimo scope planu 04-02.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Plan 04-02 je funkcne uzavreny, tooltip a inline note jsou vizualne zretelnejsi.
- Pred dalsimi kroky je vhodne zvazit samostatny plan na cleanup predexistujicich clippy chyb mimo tento scope.

---
*Phase: 04-infrastructure*
*Completed: 2026-03-05*

## Self-Check: PASSED
- Verified summary file exists.
- Verified task commits exist (`e393ebd`, `bd28751`).
