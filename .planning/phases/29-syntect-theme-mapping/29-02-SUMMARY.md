---
phase: 29-syntect-theme-mapping
plan: 02
subsystem: ui
tags: [terminal, egui, theme, contrast, regression-tests]
requires:
  - phase: 29-syntect-theme-mapping
    provides: syntect mapping and variant-aware visuals in Settings
provides:
  - Variant-aware terminal dark background mapping driven by active egui visuals
  - Regression gates for light/dark terminal background mapping and dark contrast
affects: [terminal, theme-variants, verification]
tech-stack:
  added: []
  patterns: [terminal palette derived from visuals.panel_fill, variant regression gates in theme.rs]
key-files:
  created: []
  modified: [src/app/ui/terminal/instance/theme.rs]
key-decisions:
  - "Dark terminal palette už nebere statický default, ale tónuje se podle aktivního visuals.panel_fill."
  - "Regresní test dark palety se změnil z 'must equal default palette' na kontrakt 'distinct dark variants + readable contrast'."
patterns-established:
  - "Terminal theme mapování zůstává centralizované v theme.rs bez UI override patchů."
  - "Dark režim má explicitní kontrastní guard test přes foreground/background ratio."
requirements-completed: [SYNTAX-02]
duration: 6 min
completed: 2026-03-11
---

# Phase 29 Plan 02: Terminal Background Mapping Summary

**Terminálová dark paleta je navázaná na aktivní theme variantu přes `Visuals::panel_fill` a chráněná regresními testy pro dark background i kontrast.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-10T23:23:20Z
- **Completed:** 2026-03-10T23:29:22Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Přepojeno generování dark terminal backgroundu na aktivní `egui::Visuals` místo statického fallbacku.
- Zachovaný kontrakt tmavého backgroundu i pro `DarkVariant::Midnight` bez světlého/oslňujícího pádu.
- Doplněné regresní testy pro 4 light + 2 dark variant mapping a dark foreground/background kontrast.

## Task Commits

Each task was committed atomically:

1. **Task 1: Navázat terminal background na aktivní theme variantu včetně dark variant** - `6c8f851` (test), `097dafc` (feat)
2. **Task 2: Dopsat regresní testy pro background kontrast a variant mapping** - `5c187a6` (test)

## Files Created/Modified
- `src/app/ui/terminal/instance/theme.rs` - Variant-aware dark palette tónování + rozšířená regresní test gate.

## Decisions Made
- `terminal_palette()` v dark režimu používá `tone_dark_palette(ColorPalette::default(), visuals)` pro deterministické napojení na aktivní variantu.
- Historický test ekvivalence dark palety s default `egui_term` byl odstraněn, protože blokoval požadované variant-specific chování.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `cargo` build/test selhával přes `sccache` (Operation not permitted)**
- **Found during:** Task 1 (TDD RED verification)
- **Issue:** Build skripty (`ring`) neprošly kvůli `RUSTC_WRAPPER=sccache` v tomto prostředí.
- **Fix:** Verifikační příkazy plánu (`cargo test`, `cargo check`, `./check.sh`) spuštěny s `RUSTC_WRAPPER=`.
- **Files modified:** žádné (runtime execution override)
- **Verification:** Target testy i `cargo check` doběhly korektně.
- **Committed in:** N/A (bez změny souborů)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Bez scope creep; šlo o čisté odblokování verifikace.

## Issues Encountered
- `./check.sh` stále padá na `cargo fmt` drift v souborech mimo scope plánu (`src/app/ui/git_status.rs`, `src/app/ui/workspace/modal_dialogs/settings.rs`), evidováno v `deferred-items.md`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Plan 29-02 je implementačně dokončený a regression gate je zelená.
- Fáze 29 má po doplnění této summary připravený podklad pro uzavření.

---
*Phase: 29-syntect-theme-mapping*
*Completed: 2026-03-11*

## Self-Check: PASSED
- SUMMARY file exists.
- All task commits are present in git history.
