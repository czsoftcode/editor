---
id: T03
parent: S05
milestone: M001
provides:
  - Snapshot-aware Settings modal Save/Cancel with fingerprint diff guard
  - Global discard cleanup restoring runtime settings from snapshot
  - Canonical settings.toml persistence + legacy settings.json migration tests
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 5min
verification_result: passed
completed_at: 2026-03-04
blocker_discovered: false
---
# T03: 03-light-varianty-settings-ui 03

**# Phase 3 Plan 3: Settings Persistence a Modal Lifecycle Summary**

## What Happened

# Phase 3 Plan 3: Settings Persistence a Modal Lifecycle Summary

**Snapshot-aware Settings modal Save/Cancel s fingerprint diff guardem a canonical settings.toml persistencí + legacy JSON migrací**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-04T23:32:00Z
- **Completed:** 2026-03-04T23:37:40Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Save/Cancel lifecycle je snapshot-aware: Cancel revertu runtime settings ze `settings_original`, Save persistuje na disk jen při reálné změně theme fingerprintu
- Global confirm-discard flow čistí `settings_draft` i `settings_original` a revertu runtime — žádný leak preview stavu
- 15 testů v `settings::tests` je zelených, pokrývají canonical TOML roundtrip, legacy JSON migraci, backward compat a všechny light varianty

## Task Commits

Tasky byly implementovány v předchozích iteracích tohoto plánu:

1. **Task 1: Snapshot lifecycle pro Save/Cancel při live preview** - `d6455c8` (fix)
2. **Task 2: Cleanup při global confirm-discard** - `91abd8e` (fix)
3. **Task 3: Persistence testy pro SETT-03** - `b34c2af` (test)

## Files Created/Modified

- `src/app/ui/workspace/modal_dialogs/settings.rs` — `theme_fingerprint()`, `should_persist_theme_change()`, `discard_settings_draft()`, snapshot capture při otevření modalu, Save/Cancel logika
- `src/app/ui/workspace/modal_dialogs.rs` — global discard volá `discard_settings_draft()` před zavřením modalu
- `src/app/ui/workspace/state/mod.rs` — `settings_original: Option<Settings>` field, inicializace na `None` v `init.rs`
- `src/settings.rs` — `SETTINGS_FILE`/`OLD_SETTINGS_FILE` konstanty, `load_from_config_dir()`/`save_to_config_dir()` s migrací, persistence testy

## Decisions Made

- Save persistuje na disk pouze při reálné změně theme fingerprintu `(dark_theme, light_variant)` — beze změny se `settings.toml` nepřepisuje
- Cancel/discard restoruje runtime ze snapshot a bumpne `settings_version` jen pokud se theme fingerprint liší (zbytečný repaint se vynechá)
- Global confirm-discard flow volá `discard_settings_draft()`, která zároveň revertu snapshot i čistí draft — žádný leak preview stavu
- Canonical storage je `settings.toml`; `settings.json` je pouze legacy migrační vstup, který se po migraci smaže
- Testy persistence jsou izolovány přes `TempConfigDir` s unikátním tmp adresářem — nezasahují reálný user config

## Deviations from Plan

Žádné. Implementace byla kompletní z předchozích iterací plánu 03-03. Verifikace `cargo check` a `cargo test settings::tests` proběhla bez chyb — 15/15 testů zelených.

## Issues Encountered

Žádné — vše bylo implementováno v předchozích commitech tohoto plánu.

## Next Phase Readiness

- Settings modal lifecycle je kompletní — snapshot, preview, Save/Cancel, discard flow
- Canonical TOML persistence s legacy JSON migrací je testována a funkční
- Phase 03-04 může navazovat na finalizaci světlých variant (terminal, git barvy)

---
*Phase: 03-light-varianty-settings-ui*
*Completed: 2026-03-04*

## Self-Check: PASSED

- SUMMARY.md: FOUND
- Commit d6455c8 (Task 1): FOUND
- Commit 91abd8e (Task 2): FOUND
- Commit b34c2af (Task 3): FOUND
