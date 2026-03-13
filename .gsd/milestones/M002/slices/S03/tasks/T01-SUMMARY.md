---
id: T01
parent: S03
milestone: M002
provides:
  - cleanup_history_dir standalone funkce s max_age_secs parametrem
  - background cleanup thread při startu workspace
  - invalidace history_view při zavření tabu (oba close pathy)
  - unit testy pro max_age a max_versions cleanup
key_files:
  - src/app/local_history.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/mod.rs
key_decisions:
  - Sloučení max_versions a max_age do jednoho průchodu — po seřazení nejdřív skip max_versions, pak z ponechaných kontrola stáří
  - cleanup_history_dir jako pub standalone funkce místo metody na LocalHistory — umožňuje Send-safe volání z threadu
patterns_established:
  - Standalone FS utility funkce pro operace potřebné v background threadech (cleanup_history_dir pattern)
observability_surfaces:
  - none (cleanup je fire-and-forget, chyby mazání tiché — pragmatické pro background FS operaci)
duration: ~15 min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Cleanup s max_age, edge case handling při zavření tabu a finální verifikace

**Rozšířen cleanup o max_age_secs, extrahována standalone cleanup_history_dir(), spuštěn background cleanup při startu workspace, invalidace history_view ve dvou close pathech, i18n audit kompletní.**

## What Happened

1. **`cleanup()` rozšířen o `max_age_secs: Option<u64>`** — deleguje na novou `cleanup_history_dir()`. Jednoduchý průchod: verze nad `max_versions` se mažou bezpodmínečně, verze v limitu se kontrolují proti `max_age`.

2. **`cleanup_history_dir()` extrahována** jako veřejná standalone funkce — `Send`-safe, žádný `&self`. Přijímá `(base_dir, max_versions, max_age_secs)`.

3. **Background cleanup v `init_workspace()`** — `std::thread::spawn` s `cleanup_history_dir(&cleanup_base, 50, Some(30 * 24 * 3600))`. Spouští se jednorázově při otevření workspace.

4. **Edge case: `request_close_tab_target()`** — po `close_tabs_for_path` se kontroluje, zda `history_view` odkazuje na zavíraný soubor, a pokud ano, nastaví se na `None`.

5. **Edge case: `process_unsaved_close_guard_dialog()`** — stejná invalidace `history_view` po `close_tabs_for_path` v dirty close flow.

6. **Dva nové unit testy** — `cleanup_removes_old_versions_by_age` (starý snapshot smazán, čerstvý zůstává) a `cleanup_respects_max_versions_before_age` (max_versions má přednost před age).

7. **i18n audit** — všech 11 reálně používaných klíčů (tab-context-history, tab-context-close, history-panel-*, history-nav-newer, history-nav-older, history-current-label, history-historical-label, history-version-info, error-history-snapshot) je přítomno ve všech 5 jazycích (cs, en, sk, de, ru).

## Verification

- `cargo check` — bez chyb
- `cargo clippy` — bez warningů
- `cargo test -- local_history` — 8 testů prošlo (6 existujících + 2 nové)
- `./check.sh` — 135 unit testů prošlo, 1 preexistující selhání (phase35_delete_foundation_scope_guard — chybějící soubor, nesouvisí s M002)
- i18n grep audit — 11/11 klíčů kompletních ve všech 5 jazycích

## Diagnostics

Background cleanup je fire-and-forget — žádný návratový kanál. Pokud cleanup selže (FS chyba), snapshoty zůstanou na disku — degradace je jen zbytečný disk space, ne ztráta funkcionality. Diagnostika: `ls -la .polycredo/history/*/` pro ruční inspekci.

## Deviations

- Plán zmiňoval klíče `history-nav-prev` a `history-nav-next` — ty neexistují v kódu ani v locale souborech. Skutečné klíče jsou `history-nav-newer` a `history-nav-older` (implementovány v S02). Audit upraven na reálné klíče.
- Přidán druhý test `cleanup_respects_max_versions_before_age` nad rámec plánu pro pokrytí interakce max_versions + max_age.

## Known Issues

- Preexistující test failure: `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` — chybějící soubor, nesouvisí s M002.

## Files Created/Modified

- `src/app/local_history.rs` — rozšířen `cleanup()` o `max_age_secs`, nová `cleanup_history_dir()`, 2 nové unit testy
- `src/app/ui/workspace/state/init.rs` — background cleanup thread v `init_workspace()`
- `src/app/ui/workspace/mod.rs` — invalidace `history_view` v `request_close_tab_target()` a `process_unsaved_close_guard_dialog()`
