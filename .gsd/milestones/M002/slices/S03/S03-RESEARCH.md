# S03: Cleanup, Edge Cases a Finální Integrace — Research

**Date:** 2026-03-13

## Summary

S03 je nízko-riziková slice zaměřená na čtyři oblasti: (1) cleanup retence při startu workspace, (2) edge case handling při zavírání tabů v history mode, (3) vyčištění testovacích relikvií v `.polycredo/history/`, a (4) finální i18n audit.

Stávající `cleanup()` v `local_history.rs` podporuje jen `max_versions` (počet verzí) — chybí `max_age` parametr (30 dní). Cleanup se nikde nevolá — musí být spuštěn v background threadu při `init_workspace()`. Existující data v `.polycredo/history/` jsou 63 adresářů / 153 souborů / 2.6 MB z ~5. března 2026 — testovací relikty, které cleanup automaticky vyčistí, pokud bude mít `max_age` parametr.

Hlavní edge case je zavření tabu v history mode — `close_tabs_for_path()` v `editor/tabs.rs` nemá žádnou referenci na `history_view` v `WorkspaceState`. Pokud uživatel zavře tab, pro který je otevřený split view, `history_view` zůstane viset s neplatnou referencí na soubor. Řešení: v `request_close_tab_target()` a v unsaved-close-guard flow vyčistit `history_view`, pokud odkazuje na zavíraný soubor.

## Recommendation

Rozdělit na 2 malé tasky:
- **T01:** Backend cleanup (rozšíření `cleanup()` o `max_age`, volání z `init_workspace` v background threadu, unit test) + edge case handling (vyčištění `history_view` při zavření tabu).
- **T02:** Finální i18n audit, manuální UAT scénáře, verifikace.

Alternativa — jeden task — je taky legitimní, scope je malý. Rozhodnutí při plánování.

Cleanup se musí spustit v background threadu, protože může iterovat stovky souborů. Pattern: `std::thread::spawn` s klonovanou `LocalHistory` (nebo jen `base_dir: PathBuf`), protože `LocalHistory` drží `&mut self` pro `take_snapshot` ale `cleanup` má `&self` — stačí předat `base_dir` a spustit standalone cleanup funkci.

**Problém:** `cleanup(&self)` drží referenci na `LocalHistory`, ale `LocalHistory` žije ve `WorkspaceState` a není `Send`. Řešení: extrahovat `cleanup` do standalone funkce `cleanup_history_dir(base_dir: &Path, max_versions: usize, max_age_days: u64)` nebo klonovat potřebná data (jen `base_dir: PathBuf`) a spustit v threadu.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Mazání starých souborů | `std::fs::remove_file` + iterace v `cleanup()` | Už existuje, jen rozšířit o `max_age`. |
| Background thread | `std::thread::spawn` (pattern v celém codebase) | 8+ existujících použití v `src/app/ui/`. |
| Timestamp porovnání | `SystemTime::now().duration_since(UNIX_EPOCH)` | Konzistentní se snapshot timestamp logikou. |
| i18n | Fluent `.ftl` soubory v `locales/{cs,en,sk,de,ru}/` | Standardní pattern, klíče už existují z S01+S02. |

## Existing Code and Patterns

- `src/app/local_history.rs` řádek 143–183 — `cleanup(&self, max_versions)`: iteruje `base_dir`, parsuje timestamp z filename, sortuje, maže přes limit. **Rozšířit:** přidat `max_age_secs: Option<u64>` parametr a filtrovat i podle stáří (current_ts - file_ts > max_age_secs).
- `src/app/ui/workspace/state/init.rs` řádek 17–145 — `init_workspace()`: místo pro spuštění background cleanup threadu. Po vytvoření `local_history` a před return `WorkspaceState`.
- `src/app/ui/workspace/mod.rs` řádek 282–306 — `request_close_tab_target()`: po zavření tabu je potřeba přidat check `if ws.history_view.as_ref().map_or(false, |hv| hv.file_path == target_path) { ws.history_view = None; }`.
- `src/app/ui/workspace/mod.rs` řádek 351+ — `process_unsaved_close_guard_dialog()`: po `ws.editor.close_tabs_for_path()` přidat stejný history_view cleanup.
- `src/app/ui/workspace/mod.rs` řádek 665–735 — CentralPanel rendering: `ws.history_view` se kontroluje, ale nikdy se neinvaliduje mimo zavírací tlačítko ✕. Edge case: pokud tab zmizí (external delete), `history_view` zůstane viset.
- `src/watcher.rs` řádek 203–204 — `.polycredo` filtr potvrzený ✓. Cleanup nebude triggerovat watcher eventy.

## Constraints

- **`LocalHistory` není `Send`** — obsahuje `HashMap` a `PathBuf`, ale `&self` reference nelze přesunout do threadu. Pro background cleanup musí být data klonována nebo extrahována do standalone funkce.
- **`cargo check` + `./check.sh` musí projít** — 133 testů + 1 preexistující selhání (phase35_delete_foundation, nesouvisí).
- **Žádné nové runtime závislosti.**
- **Cleanup nesmí blokovat UI** — musí běžet v `std::thread::spawn`.
- **i18n: 5 jazyků** (cs, en, sk, de, ru) — S01+S02 přidaly celkem 12 klíčů do `ui.ftl` + 1 klíč do `errors.ftl` pro každý jazyk. S03 pravděpodobně nepřidá nové klíče (pokud ne, audit je jen verifikace).
- **Stávající testovací data** — 63 adresářů, 153 snapshot souborů, 2.6 MB. Cleanup s `max_age = 30 dní` vymaže vše starší 8 dní (data z 5.3.2026, dnes 13.3.2026). Pro kompletní vyčištění je potřeba buď: (a) počkat 30 dní, nebo (b) smazat manuálně/programaticky při prvním spuštění nové verze. **Doporučení:** cleanup to vyřeší automaticky — `max_versions = 50` je dostatečný pro retenci, `max_age = 30 dní` smaže staré po 30 dnech. Data z 5.3. budou smazána po 4.4.2026. Pokud je potřeba okamžité vyčištění, přidat jednorázový "nuke" v init, ale to není nutné.

## Common Pitfalls

- **History view viset na zavřeném tabu** — `close_tabs_for_path()` žije v `Editor` structu, který nemá přístup k `history_view` ve `WorkspaceState`. Cleanup musí probíhat na workspace úrovni, po volání `close_tabs_for_path`. Dvě místa: `request_close_tab_target()` (clean tab close) a `process_unsaved_close_guard_dialog()` (dirty tab close).
- **Cleanup I/O chyby** — `cleanup()` aktuálně ignoruje všechny I/O chyby (`let _ = fs::remove_file`). Pro S-3 compliance by měl logovat chyby, ale ne toastovat — cleanup je background operace, ne uživatelská akce. Ponechat `let _` je pragmatické.
- **Race condition: cleanup běží v background threadu, snapshot přichází z UI threadu** — `take_snapshot()` drží `&mut self`, cleanup pracuje přímo na FS. Pokud cleanup maže soubor, který právě `get_snapshot_content()` čte, dostaneme I/O error. **Mitigace:** cleanup běží jen při startu (jednorázově), snapshoty začínají až po inicializaci UI — timing conflict je nepravděpodobný.
- **`max_age` timestamp vs filesystem mtime** — Cleanup parsuje timestamp z filename (UNIX epoch seconds), ne z filesystem mtime. To je správné — filename timestamp je authoritative, mtime se může změnit kopírováním.

## Open Risks

- **Žádné high-risk položky** — S03 je low-risk slice. Všechny operace jsou přímočaré FS manipulace a UI stav cleanup.
- **Preexistující test failure** — `phase35_delete_foundation` selhává na chybějící soubor. Nesouvisí s M002, ale zkresluje `./check.sh` výstup. Ignorovat, ne opravovat.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | — | None found (žádné relevantní skills pro low-level UI state cleanup) |
| Rust desktop | — | Not needed for this scope (standalone FS operations + state management) |

## Sources

- Codebase exploration: `local_history.rs`, `workspace/mod.rs`, `background.rs`, `init.rs`, `tabs.rs`, `watcher.rs`
- `.polycredo/history/` filesystem inspection (63 dirs, 153 files, 2.6 MB, timestamps 2026-03-05)
- S01-SUMMARY.md, S02-SUMMARY.md — forward intelligence pro edge cases a fragile body
