---
estimated_steps: 7
estimated_files: 5
---

# T01: Cleanup s max_age, edge case handling při zavření tabu a finální verifikace

**Slice:** S03 — Cleanup, Edge Cases a Finální Integrace
**Milestone:** M002

## Description

Rozšířit cleanup o max_age parametr a spustit ho v background threadu při startu workspace. Ošetřit edge case zavření tabu v history mode (oba close pathy). Ověřit i18n kompletnost a celkovou stabilitu.

## Steps

1. **Rozšířit `cleanup()` v `local_history.rs`** — přidat parametr `max_age_secs: Option<u64>`. Po stávajícím skip max_versions přidat druhou iteraci přes zbylé verze: pokud `current_ts - file_ts > max_age_secs`, smazat. Alternativně sloučit do jednoho průchodu — po seřazení nejdřív skipmout max_versions, pak ze zbylých filtrovat i ty, co jsou v limitu verzí ale překročily max_age.
2. **Extrahovat standalone `cleanup_history_dir()`** — veřejná funkce s parametry `(base_dir: &Path, max_versions: usize, max_age_secs: Option<u64>)` která obsahuje samotnou logiku. `cleanup(&self, ...)` na ni deleguje. Tato funkce je `Send`-safe (žádný `&self`).
3. **Spustit background cleanup v `init_workspace()`** — po řádku `local_history: LocalHistory::new(&root_path)` klonovat `root_path` a spustit `std::thread::spawn` s voláním `cleanup_history_dir()`. Parametry: `max_versions = 50`, `max_age_secs = Some(30 * 24 * 3600)`.
4. **Edge case: `request_close_tab_target()`** — po volání `ws.editor.close_tabs_for_path(&target_path)` (řádek ~183) přidat: pokud `ws.history_view` odkazuje na `target_path`, nastavit na `None`.
5. **Edge case: `process_unsaved_close_guard_dialog()`** — po `ws.editor.close_tabs_for_path(&current_path)` (řádek ~389) přidat stejnou podmínku: pokud `ws.history_view` odkazuje na `current_path`, nastavit na `None`.
6. **Unit test** — test `cleanup_removes_old_versions_by_age` v `local_history.rs`: vytvořit snapshoty se starým timestampem (např. 60 dní), spustit cleanup s `max_age_secs = Some(30 * 24 * 3600)`, ověřit smazání.
7. **Verifikace a i18n audit** — `cargo check`, `cargo clippy`, `./check.sh`. Grep i18n klíčů z S01+S02 (`tab-context-history`, `tab-context-close`, `history-panel-*`, `history-nav-*`, `history-current-label`, `history-historical-label`, `history-version-info`, `error-history-snapshot`) ve všech 5 locale souborech.

## Must-Haves

- [ ] `cleanup()` / `cleanup_history_dir()` maže verze starší než max_age
- [ ] Background thread v `init_workspace()` spouští cleanup
- [ ] `history_view = None` při clean close tabu (request_close_tab_target)
- [ ] `history_view = None` při dirty close tabu (process_unsaved_close_guard_dialog)
- [ ] Unit test pro max_age cleanup
- [ ] i18n klíče kompletní ve všech 5 jazycích
- [ ] `cargo check` + `./check.sh` prochází

## Verification

- `cargo test -- local_history` — všechny testy prochází (existující 6 + nový 1)
- `cargo check` — bez chyb
- `cargo clippy` — bez nových warningů
- `./check.sh` — 133+ testů (preexistující phase35 ignorovat)
- Grep audit: každý i18n klíč se nachází ve všech 5 locale souborech

## Inputs

- `src/app/local_history.rs` — stávající `cleanup(&self, max_versions)` na řádku 165
- `src/app/ui/workspace/state/init.rs` — `init_workspace()` s `local_history` na řádku 135
- `src/app/ui/workspace/mod.rs` — `request_close_tab_target()` řádek 157, `process_unsaved_close_guard_dialog()` řádek 281
- S01/S02 summaries — seznam i18n klíčů k auditování
- S03-RESEARCH.md — constraints (LocalHistory není Send, standalone funkce nutná)

## Expected Output

- `src/app/local_history.rs` — rozšířený `cleanup()` s `max_age_secs`, nová `cleanup_history_dir()`, nový unit test
- `src/app/ui/workspace/state/init.rs` — `thread::spawn` pro background cleanup
- `src/app/ui/workspace/mod.rs` — history_view invalidace ve dvou close pathech
