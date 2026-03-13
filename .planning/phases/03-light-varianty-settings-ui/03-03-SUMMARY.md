---
phase: 03-light-varianty-settings-ui
plan: "03"
subsystem: ui
tags: [settings, persistence, toml, snapshot, modal, theme, rust]

# Dependency graph
requires:
  - phase: 03-02
    provides: live preview guarded settings modal with settings_draft lifecycle
provides:
  - Snapshot-aware Settings modal Save/Cancel with fingerprint diff guard
  - Global discard cleanup restoring runtime settings from snapshot
  - Canonical settings.toml persistence + legacy settings.json migration tests
affects: [03-04]

# Tech tracking
tech-stack:
  added: []
  patterns: [snapshot-based modal lifecycle, theme fingerprint diff guard, TOML canonical / JSON legacy migration]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/workspace/modal_dialogs.rs
    - src/app/ui/workspace/state/mod.rs
    - src/settings.rs

key-decisions:
  - "Save persistuje na disk pouze při reálné změně theme fingerprintu (dark_theme, light_variant) — beze změny se settings.toml nepřepisuje."
  - "Cancel/discard restoruje runtime settings ze snapshot a bumpne settings_version jen pokud se theme fingerprint liší."
  - "Global confirm-discard flow volá discard_settings_draft(), která zároveň revertu snapshot i čistí draft — žádný leak preview stavu."
  - "Canonical storage je settings.toml; settings.json je pouze legacy migrační vstup, který se po migraci smaže."
  - "Testy persistence jsou izolovány přes TempConfigDir s unikátním tmp adresářem — nezasahují reálný user config."

patterns-established:
  - "Modal lifecycle pattern: settings_original snapshot on open → apply_theme_preview on change → restore from snapshot on cancel/discard → clear both on save"
  - "Fingerprint diff guard: persist only when (dark_theme, light_variant) tuple changes between snapshot and draft"

requirements-completed: [SETT-03, SETT-02]

# Metrics
duration: 5min
completed: 2026-03-04
---

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
