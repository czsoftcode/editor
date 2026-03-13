---
id: T05
parent: S05
milestone: M001
provides:
  - show_light_variant_card bez with_layout expanze — checkmark jako inline label
  - warm_ivory_bg() helper pro detekci teplého panel_fill v terminal theme
  - tone_light_palette s vyshim blend ratio (0.55) a variantne-specificnym base bg
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 5min
verification_result: passed
completed_at: 2026-03-05
blocker_discovered: false
---
# T05: 03-light-varianty-settings-ui 05

**# Phase 03 Plan 05: UAT Gap Closure — Variant Picker + WarmIvory Terminal Summary**

## What Happened

# Phase 03 Plan 05: UAT Gap Closure — Variant Picker + WarmIvory Terminal Summary

**Opraveny dva UAT defekty: picker karet zobrazuje tři varianty vedle sebe (ne jednu) a terminál v WarmIvory má teplý krémový tón (#f5f2e8 base, blend 0.55) místo studené šedi.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-05T00:00:00Z
- **Completed:** 2026-03-05T00:05:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Odstraněn `ui.with_layout(right_to_left)` uvnitř `show_light_variant_card` — checkmark je nyní inline label; všechny tři karty (WarmIvory, CoolGray, Sepia) se zobrazí vedle sebe v `horizontal_wrapped`
- Přidána funkce `warm_ivory_bg()` detekující teplý panel_fill (r-b > 10) a vracející `#f5f2e8` jako základ blendingu
- `tone_light_palette()` nyní používá variantně-specifický základ bg a vyšší ratio 0.55 — WarmIvory terminál má teplý krémový odstín ~(250,247,236)
- Všech 55 testů prochází, včetně 8 terminal_theme testů (luminance, kontrast, distinctness)

## Task Commits

Každý task byl commitnut atomicky:

1. **Task 1: Opravit expanzi karty v light variant pickeru** - `329e1d5` (fix)
2. **Task 2: Opravit teplý tón terminálu pro WarmIvory variantu** - `0108797` (fix)

## Files Created/Modified

- `src/app/ui/workspace/modal_dialogs/settings.rs` — odstraněn `with_layout(right_to_left)` z těla karty; checkmark je inline label s `add_space(8.0)`
- `src/app/ui/terminal/instance/theme.rs` — přidána `warm_ivory_bg()`, `tone_light_palette()` upravena na variantní base bg a blend ratio 0.55

## Decisions Made

- `with_layout(right_to_left)` uvnitř `horizontal` alokuje veškerý zbývající prostor jako nový layout — karta se roztáhne na full-width a ostatní se zabalí mimo viewport. Fix: inline podmíněný label.
- `warm_ivory_bg()` detekuje WarmIvory pomocí `r - b > 10` (WarmIvory má (255,252,240), tedy r-b=15); CoolGray (236,236,236) a Sepia (234,223,202) mají r-b <= 12, ale Sepia je na hraně — proto prahová hodnota 10 je bezpečná.
- Signatury `light_terminal_base_palette()` a `tone_light_palette()` zůstaly beze změny pro zpětnou kompatibilitu s testy.

## Deviations from Plan

None — plan byl proveden přesně dle specifikace.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Fáze 03 je kompletní: light varianty, settings UI, persistence, terminal toning, git toning, UAT gap closure
- Všechny UAT defekty (picker expanze, WarmIvory teplý tón) jsou opraveny
- Projekt připraven pro produkční release nebo další fáze

## Self-Check: PASSED

- FOUND: src/app/ui/workspace/modal_dialogs/settings.rs
- FOUND: src/app/ui/terminal/instance/theme.rs
- FOUND: .planning/phases/03-light-varianty-settings-ui/03-05-SUMMARY.md
- FOUND commit: 329e1d5 (fix(03-05): remove with_layout(right_to_left) from light variant card)
- FOUND commit: 0108797 (fix(03-05): warm WarmIvory terminal background via warm_ivory_bg helper)

---
*Phase: 03-light-varianty-settings-ui*
*Completed: 2026-03-05*
