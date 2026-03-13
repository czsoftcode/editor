---
id: S03
parent: M002
milestone: M002
provides:
  - cleanup_history_dir() standalone funkce s max_age_secs parametrem — spustitelná v background threadu
  - background cleanup thread při startu workspace (max 50 verzí, max 30 dní)
  - invalidace history_view při zavření tabu (clean close i dirty close)
  - unit testy pro max_age a max_versions interakci
  - kompletní i18n audit — 10 klíčů ve všech 5 jazycích
requires:
  - slice: S01
    provides: LocalHistory s cleanup() metodou, snapshot pipeline, HistoryViewState
  - slice: S02
    provides: split view rendering, diff cache, navigace šipkami
affects: []
key_files:
  - src/app/local_history.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/mod.rs
key_decisions:
  - "cleanup_history_dir() jako standalone funkce bez &self — LocalHistory není Send, pro thread::spawn nutné extrahovat logiku do volné funkce s PathBuf parametrem"
  - "Cleanup I/O chyby ponechány jako let _ — background operace při startu, ne uživatelská akce, toastování by bylo rušivé"
  - "S03 jako jeden task — scope je malý (cleanup rozšíření, 2× edge case fix, i18n audit), žádná nová UI komponenta"
patterns_established:
  - Standalone FS utility funkce pro background thread operace (cleanup_history_dir pattern)
observability_surfaces:
  - none (cleanup je fire-and-forget, degradace = zbytečný disk space)
drill_down_paths:
  - .gsd/milestones/M002/slices/S03/tasks/T01-SUMMARY.md
duration: ~15 min
verification_result: passed
completed_at: 2026-03-13
---

# S03: Cleanup, Edge Cases a Finální Integrace

**Retence snapshotu (max 50 verzí / 30 dní) běží automaticky v background threadu při startu workspace. Zavření tabu v history mode čistí stav. i18n kompletní.**

## What Happened

Slice S03 uzavírá milestone M002 třemi koherentními změnami:

1. **Cleanup retence** — existující `cleanup()` metoda rozšířena o `max_age_secs: Option<u64>` parametr. Logika extrahována do standalone `cleanup_history_dir(base_dir, max_versions, max_age_secs)` funkce, která je `Send`-safe a spustitelná z threadu. V `init_workspace()` se po vytvoření `local_history` spustí `std::thread::spawn` s cleanup konfigurací 50 verzí / 30 dní.

2. **Edge case: zavření tabu v history mode** — dvě cesty (clean close přes `request_close_tab_target()` a dirty close přes `process_unsaved_close_guard_dialog()`) nyní kontrolují, zda `history_view` odkazuje na zavíraný soubor, a pokud ano, nastaví ho na `None`. Bez tohoto by history view zůstal viset po zavření zdrojového tabu.

3. **i18n audit** — 10 reálně používaných klíčů (tab-context-history, tab-context-close, history-panel-title, history-panel-no-versions, history-nav-newer, history-nav-older, history-current-label, history-historical-label, history-version-info, error-history-snapshot) ověřeno kompletních ve všech 5 jazycích (cs, en, sk, de, ru).

## Verification

- `cargo check` — bez chyb
- `cargo clippy` — bez warningů
- `cargo test` — 8/8 local_history testů prošlo (6 z S01 + 2 nové z S03: `cleanup_removes_old_versions_by_age`, `cleanup_respects_max_versions_before_age`)
- `./check.sh` — 148 testů prošlo, 1 preexistující selhání (`phase35_delete_foundation_scope_guard` — chybějící soubor, nesouvisí s M002)
- i18n grep audit — 10/10 klíčů kompletních ve všech 5 jazycích

## Deviations

- Plán (T01 summary) uváděl 11 i18n klíčů včetně `history-panel-no-workspace` — ten se v kódu nepoužívá, audit upraven na 10 reálných klíčů.

## Known Limitations

- Background cleanup je fire-and-forget — žádný reporting o výsledku. Selhání cleanup (FS chyba) znamená, že staré snapshoty zůstanou na disku. Degradace je jen zbytečný disk space.
- Preexistující test `phase35_delete_foundation_scope_guard` stále selhává (chybějící soubor z v1.3.1, mimo scope M002).

## Follow-ups

- none

## Files Created/Modified

- `src/app/local_history.rs` — rozšířen `cleanup()` o `max_age_secs`, nová `cleanup_history_dir()`, 2 nové unit testy
- `src/app/ui/workspace/state/init.rs` — background cleanup thread v `init_workspace()`
- `src/app/ui/workspace/mod.rs` — invalidace `history_view` v `request_close_tab_target()` a `process_unsaved_close_guard_dialog()`

## Forward Intelligence

### What the next slice should know
- M002 je kompletní. Následující práce by měla cílit na robustnostní backlog (V-1, V-2, K-1, S-3 zbytek, S-1).

### What's fragile
- `cleanup_history_dir()` parsuje timestamp z názvu souboru (`_` split, pozice 0) — pokud se změní formát pojmenování snapshotů, cleanup přestane mazat staré verze.

### Authoritative diagnostics
- `ls -la .polycredo/history/*/` — přímá inspekce snapshot retence na disku
- `cargo test -- local_history` — 8 testů pokrývajících snapshot vytvoření, duplikáty, cleanup max_versions, cleanup max_age, readonly FS

### What assumptions changed
- Plán předpokládal klíč `history-panel-no-workspace` — ten neexistuje v kódu ani locale souborech. Nebylo třeba ho přidávat.
