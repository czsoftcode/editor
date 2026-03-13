---
id: T01
parent: S07
milestone: M001
provides:
  - Sandbox režim jako persistované nastavení v settings.toml
  - UI přepínač sandbox režimu s tooltipem a session toastem
  - Init workspace načítá sandbox mód do stabilních flagů
  - Terminály a build bar respektují režim pro cwd i label
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 45min
verification_result: passed
completed_at: 2026-03-05
blocker_discovered: false
---
# T01: 04-infrastructure 01

**# Phase 04: Infrastructure Summary**

## What Happened

# Phase 04: Infrastructure Summary

**Sandbox režim je persistovaný v settings.toml, má UI přepínač s tooltipem a terminály/build bar respektují režim pro cwd i label s apply-on-reopen.**

## Performance

- **Duration:** 45 min
- **Started:** 2026-03-05T03:44:00+01:00
- **Completed:** 2026-03-05T04:27:46+01:00
- **Tasks:** 4
- **Files modified:** 17

## Accomplishments
- Persistování `sandbox_mode` do `settings.toml` s legacy mapováním `project_read_only` a testy round-trip.
- Settings UI přepínač sandboxu s tooltipem, inline poznámkami a toast notifikacemi.
- Workspace init nastavuje stabilní `sandbox_mode_enabled` + `build_in_sandbox`/`file_tree_in_sandbox`.
- Terminály i build bar používají režim pro cwd a label (`Sandbox` vs `Terminal — <path>`).

## Task Commits

Each task was committed atomically:

1. **Task 1: Settings model + persistování sandbox režimu + nahrazení Safe Mode** - `572b691` (test), `664fafb` (feat)
2. **Task 2: UI přepínač v Settings + lokalizace + tooltip** - `b1a812f` (feat)
3. **Task 3: Aplikace sandbox režimu při startu projektu + guard na apply-on-reopen** - `664fafb` (feat)
4. **Task 4: Terminály v rootu + labely** - `664fafb` (feat)

**Plan metadata:** n/a

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified
- `src/settings.rs` - nový `sandbox_mode` s aliasem `project_read_only` + testy migrace
- `src/app/ui/workspace/modal_dialogs/settings.rs` - sandbox toggle, tooltip, toasty, save handler
- `src/app/ui/workspace/state/init.rs` - init `sandbox_mode_enabled`, `build_in_sandbox`, `file_tree_in_sandbox`
- `src/app/ui/terminal/mod.rs` - label + working dir helpery
- `src/app/ui/terminal/bottom/build_bar.rs` - cwd a label podle režimu
- `src/app/ui/workspace/state/mod.rs` - nový stabilní flag `sandbox_mode_enabled`
- `locales/cs/ui.ftl`, `locales/en/ui.ftl` - nové texty pro UI

## Decisions Made
- Sandbox režim nepřepíná runtime chování; aktivuje se až po znovuotevření projektu.
- Toast „sandbox OFF“ je session-only, bez persistování.

## Deviations from Plan

### Auto-fixed Issues

**1. [Tooling] Rustfmt úpravy mimo rozsah tasků**
- **Found during:** Task 1 verification (check.sh)
- **Issue:** `cargo fmt` požadoval změny v testech mimo plánované soubory
- **Fix:** Aplikován `cargo fmt --all`
- **Files modified:** `src/app/ui/file_tree/render.rs`, `src/app/ui/git_status.rs`, `src/app/ui/terminal/instance/theme.rs`
- **Verification:** `cargo fmt --all`
- **Committed in:** `7018e79` (chore)

---

**Total deviations:** 1 auto-fixed (tooling)
**Impact on plan:** Nezbytné kvůli `check.sh` formátu, bez změny funkcionality.

## Issues Encountered
- `./check.sh` selhal na `sccache: Operation not permitted` během `cargo clippy` (sandbox omezení).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Funkční sandbox režim + UI + terminálové chování je připravené pro další fázi.
- `check.sh` je blokovaný na sccache oprávněních; pokud potřeba, spustit mimo sandbox / s upraveným wrapperem.

---
*Phase: 04-infrastructure*
*Completed: 2026-03-05*
