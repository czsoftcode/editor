# S03: Cleanup, Edge Cases a Finální Integrace

**Goal:** Cleanup retence při startu workspace funguje (max 50 verzí, max 30 dní), zavření tabu v history mode vyčistí stav, i18n je kompletní, milestoneové UAT prochází.
**Demo:** Po spuštění editoru s workspace, kde `.polycredo/history/` obsahuje staré snapshoty, se v background threadu automaticky spustí cleanup. Zavření tabu v history mode vrátí editor do normálu. `cargo check` + `./check.sh` prochází.

## Must-Haves

- `cleanup()` rozšířen o `max_age_secs: Option<u64>` — maže verze starší než zadaný limit.
- Standalone `cleanup_history_dir()` funkce bez závislosti na `&self` — spustitelná v background threadu.
- Background cleanup thread v `init_workspace()` po vytvoření `local_history`.
- `history_view` se nastaví na `None` při zavření tabu v `request_close_tab_target()` (clean close).
- `history_view` se nastaví na `None` při zavření tabu v `process_unsaved_close_guard_dialog()` (dirty close).
- Unit test pro `max_age` filtrování v cleanup.
- i18n klíče z S01+S02 jsou kompletní ve všech 5 jazycích (audit).
- `cargo check` + `./check.sh` prochází.

## Verification

- `cargo test -- local_history` — všechny testy prochází včetně nového testu na max_age cleanup
- `cargo check` — kompilace bez chyb
- `cargo clippy` — žádné nové warningy
- `./check.sh` — 133+ testů prochází (plus preexistující phase35 selhání)
- Ruční kontrola: `grep -c` všech i18n klíčů přidaných v S01+S02 ve všech 5 locale souborech

## Tasks

- [x] **T01: Cleanup s max_age, edge case handling při zavření tabu a finální verifikace** `est:30m`
  - Why: Jediný task pro celou slice — scope je malý a koherentní (FS cleanup, UI state cleanup, verifikace). Žádný krok nezávisí na výstupu jiného tasku.
  - Files: `src/app/local_history.rs`, `src/app/ui/workspace/state/init.rs`, `src/app/ui/workspace/mod.rs`, `locales/*/ui.ftl`, `locales/*/errors.ftl`
  - Do: (1) Rozšířit `cleanup()` o `max_age_secs` parametr — po skip max_versions filtrovat zbylé podle stáří. (2) Extrahovat standalone `cleanup_history_dir(base_dir, max_versions, max_age_secs)` funkci pro thread. (3) Spustit `std::thread::spawn` s cleanup v `init_workspace()`. (4) Přidat history_view invalidaci v `request_close_tab_target()` po `close_tabs_for_path`. (5) Přidat history_view invalidaci v `process_unsaved_close_guard_dialog()` po `close_tabs_for_path`. (6) Unit test pro max_age. (7) i18n audit grep.
  - Verify: `cargo test -- local_history` + `cargo check` + `./check.sh` + i18n grep audit
  - Done when: Cleanup se spustí v background threadu při init, edge case handling funguje pro oba close pathy, všechny testy prochází, i18n kompletní.

## Files Likely Touched

- `src/app/local_history.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/workspace/mod.rs`
