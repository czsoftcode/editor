---
id: T03
parent: S08
milestone: M001
provides:
  - Blokace sandbox OFF při staged souborech s dialogem pro vyřešení
  - Sync dialog při zapnutí sandbox ON s automatickým přenosem z projektu
  - Staged bar viditelná v OFF režimu dokud není staged vyřešen
  - Kompletní i18n pokrytí sandbox-staged/sync/off klíčů pro cs, en, sk, de, ru
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 15min
verification_result: passed
completed_at: 2026-03-05
blocker_discovered: false
---
# T03: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 03

**# Phase 05 Plan 03: Staged/Sync UX pro Sandbox přepínání — Summary**

## What Happened

# Phase 05 Plan 03: Staged/Sync UX pro Sandbox přepínání — Summary

**Blokace sandbox OFF při staged souborech s dialogem a automatická nabídka sync projektu do sandboxu při zapnutí ON.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-05T06:15:00Z
- **Completed:** 2026-03-05T06:30:00Z
- **Tasks:** 2 (+ 1 i18n fix)
- **Files modified:** 8

## Accomplishments

- Sandbox OFF je blokováno při staged souborech — draft se vrátí na original, show_sandbox_staged = true otevře dialog
- Staged bar zůstává viditelná v OFF režimu dokud jsou staged soubory nevyřešené
- Při zapnutí ON se zobrazí sync dialog (sandbox_sync_confirmation) s plánem přenosu z projektu do sandboxu
- Sync operace běží asynchronně přes spawn_task, výsledek se zobrazí v toast notifikaci
- Doplněna chybějící i18n klíče (27 klíčů) pro sk, de, ru — opraveno selhávající `all_lang_keys_match_english` test

## Task Commits

1. **Task 1: Staged blokace OFF + dialog** - `38021cb` (feat)
2. **Task 2: Sync při ON** - `2981d00` (feat)
3. **Deviation Fix: i18n klíče sk, de, ru** - `249c2eb` (fix)

## Files Created/Modified

- `src/app/ui/workspace/modal_dialogs/settings.rs` — should_block_sandbox_off_due_to_staged guard + blokace draftu
- `src/app/ui/workspace/mod.rs` — sandbox_sync_confirmation v process_pending_sandbox_apply
- `src/app/ui/workspace/modal_dialogs/sandbox.rs` — sandbox sync dialog modal
- `locales/cs/ui.ftl` — sandbox-off/sync klíče (cs)
- `locales/en/ui.ftl` — sandbox-off/sync klíče (en)
- `locales/sk/ui.ftl` — doplněno 27 chybějících klíčů
- `locales/de/ui.ftl` — doplněno 27 chybějících klíčů
- `locales/ru/ui.ftl` — doplněno 27 chybějících klíčů

## Decisions Made

- Blokace OFF se provádí na úrovni settings modal po záznamu sandbox_mode_change — draft se vrátí na original, zobrazí se staged dialog
- Staged bar zůstává viditelná bez ohledu na aktuální sandbox_mode_enabled, pokud sandbox_staged_files není prázdný
- Sync dialog při ON se realizuje přes sandbox_sync_confirmation: Option<SyncPlan> nastavené v process_pending_sandbox_apply
- Sync spouštěn přes spawn_task v background threadu — UI vlákno se neblokuje

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Doplněny chybějící i18n klíče pro sk, de, ru**
- **Found during:** Verifikace po task 2 (cargo test)
- **Issue:** Předchozí commity 38021cb a 2981d00 přidaly nové i18n klíče do cs/en, ale sk, de, ru je nedostaly — test `all_lang_keys_match_english` selhal s 27 chybějícími klíči pro sk
- **Fix:** Přidány sandbox-off-*, sandbox-apply-*, sandbox-persist-*, sandbox-remap-*, sandbox-sync-*, settings-conflict-* klíče do sk, de, ru s příslušnými překlady
- **Files modified:** locales/sk/ui.ftl, locales/de/ui.ftl, locales/ru/ui.ftl
- **Verification:** cargo test — 71 passed, 0 failed
- **Committed in:** 249c2eb

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Nezbytná oprava pro korektní testovací pokrytí. Žádný scope creep.

## Issues Encountered

Oba tasky (Task 1 a Task 2) byly již implementovány v předchozích commitech (38021cb, 2981d00) v rámci dřívějšího session. Plán byl v tomto běhu verifikován a doplněny chybějící i18n klíče.

## Next Phase Readiness

- Staged/sync UX flow je kompletní pro sandbox přepínání
- Všechny i18n klíče jsou pokryté pro cs, en, sk, de, ru
- Phase 05 je připravena na uzavření — všechny 3 plány dokončeny

## Self-Check

- [x] SUMMARY.md vytvořen
- [x] Commity existují: 38021cb, 2981d00, 249c2eb
- [x] cargo check: ok
- [x] cargo test: 71 passed, 0 failed

---
*Phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu*
*Completed: 2026-03-05*
