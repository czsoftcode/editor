---
id: S01
parent: M002
milestone: M002
provides:
  - Funkční background IO kanál (background_io_tx/rx propojený přes mpsc::channel)
  - Save hooky (manual, autosave, unsaved-close-guard) odesílají FsChangeResult::LocalHistory pro ne-binární taby
  - take_snapshot() vrací Result<Option<PathBuf>, io::Error> — I/O chyby se propagují do toastu
  - get_snapshot_content(rel_path, entry) -> io::Result<String> pro čtení historických verzí
  - get_history(rel_path) -> Vec<HistoryEntry> seřazený od nejnovější
  - Context menu na tab bar (pravý klik → "Historie souboru" / "Zavřít tab")
  - TabBarAction::ShowHistory(usize) varianta
  - HistoryViewState struct se stavem history panelu (file_path, entries, selected_index, preview_content)
  - render_history_panel() s horizontálním split layoutem (seznam verzí + monospace náhled)
  - Toast propagace I/O chyb z background snapshot handleru
  - i18n klíče pro context menu a history panel ve všech 5 jazycích (cs, en, sk, de, ru)
  - 6 unit testů pokrývajících snapshot pipeline
requires: []
affects:
  - S02
key_files:
  - src/app/local_history.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/background.rs
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/widgets/tab_bar.rs
  - src/app/ui/editor/render/tabs.rs
  - src/app/ui/editor/ui.rs
  - src/app/ui/terminal/right/mod.rs
  - src/app/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
  - locales/cs/errors.ftl
  - locales/en/errors.ftl
  - locales/de/errors.ftl
  - locales/ru/errors.ftl
  - locales/sk/errors.ftl
key_decisions:
  - Helper send_snapshot_signal() extrahován pro manual save; autosave a unsaved-close-guard hook inline kvůli borrow checker konfliktům
  - take_snapshot() mění signature z Option na Result — breaking change pro volající, nutné pro S-3 (I/O error propagace)
  - Timestamp formátování bez chrono dependency — vlastní days_to_date() algoritmus (Howard Hinnant) v UTC
  - History panel renderován jako overlay v CentralPanel po editor renderingu, ne jako samostatný egui::Window
  - Context menu na selectable_label response (r.context_menu) — egui nativní API pro right-click menu
  - background_io_tx se drží ve WorkspaceState, ne v Editor structu — konzistentní s I/O concerns oddělením
patterns_established:
  - Snapshot signál po úspěšném save s guardem na is_binary a strip_prefix pro relativní cestu
  - take_snapshot() vrací Result pro propagaci I/O chyb místo tichého swallowingu
  - Tab context menu přes r.context_menu() s i18n texty a guardem na is_binary
  - HistoryViewState jako Option<> pole v WorkspaceState — None = panel nezobrazen, Some = aktivní
  - render_history_panel() přijímá split reference pro partial borrow compatibility
observability_surfaces:
  - Toast při I/O chybě snapshotování (i18n klíč error-history-snapshot s parametry path a reason)
  - Snapshot soubory na FS v .polycredo/history/ jako inspectable artifact
  - Toast "Žádné historické verze" při pokusu o otevření historie pro soubor bez snapshotu
  - Chybová hláška v preview panelu při selhání čtení snapshot souboru z disku
drill_down_paths:
  - .gsd/milestones/M002/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S01/tasks/T02-SUMMARY.md
duration: 50min
verification_result: passed
completed_at: 2026-03-13
---

# S01: Snapshot Pipeline a Tab Context Menu

**Funkční snapshot pipeline propojený do save hooků s tab context menu, history panelem (seznam verzí + náhled textu) a I/O error propagací do toastu — základ pro split view v S02.**

## What Happened

Oživení mrtvého background IO kanálu a kompletní propojení save flow na snapshot pipeline s UI pro prohlížení historie.

**T01 (backend pipeline):** Vytvořen `mpsc::channel()` v `init_workspace`, sender uložen jako `background_io_tx` ve `WorkspaceState`, receiver do `background_io_rx`. Tři save hooky (manual, autosave, unsaved-close-guard) po úspěšném uložení odesílají `FsChangeResult::LocalHistory` pro ne-binární taby. `take_snapshot()` změnil signaturu na `Result<Option<PathBuf>, io::Error>` — I/O chyby se propagují a background handler je zobrazí v toastu. Přidána `get_snapshot_content()` metoda pro čtení historických verzí z disku. 6 unit testů pokrývá snapshot pipeline.

**T02 (UI + i18n):** Rozšířen `TabBarAction` o `ShowHistory(usize)`. Přidáno context menu na tab bar přes `r.context_menu()` — "Historie souboru" (jen pro ne-binární taby) a "Zavřít tab". Vytvořen modul `workspace/history/mod.rs` s `HistoryViewState` a `render_history_panel()` — horizontální split: levý panel (30%) se seznamem verzí, pravý (70%) s monospace náhledem textu. Vlastní UTC timestamp formátování bez chrono dependency (Howard Hinnant algoritmus). i18n klíče přidány do všech 5 locale souborů.

## Verification

- `cargo test -- local_history` — **6/6 testů prošlo** (take_snapshot_creates_file_on_fs, duplicate_content_is_skipped, polycredo_path_is_skipped, get_snapshot_content_returns_correct_data, get_history_returns_sorted_entries, error_on_readonly_directory)
- `cargo check` — kompilace bez chyb
- `cargo clippy` — žádné warningy
- `./check.sh` — 128 unit testů prochází, 1 pre-existující integrační test fail (phase35_delete_foundation — chybějící soubor z v1.3.1, nesouvisí s S01)
- Manuální ověření: vyžaduje spuštění GUI — pokryto UAT scénáři v S01-UAT.md

## Deviations

- **Inline snapshot hook v autosave a unsaved-close-guard:** Plán předpokládal sdílený helper pro všechny save hooky. Borrow checker neumožňuje předat `&ws` když `flow` drží `&mut ws.pending_close_flow`, proto dva hooky mají inline implementaci. Funkčně ekvivalentní.
- **Timestamp v UTC místo lokálního času:** Přidání chrono/libc pro timezone by bylo zbytečné pro jednu funkci. Pro porovnávání verzí je UTC dostatečné.
- **History panel jako overlay:** Plán zmiňoval obě varianty (overlay vs nahrazení editoru). Zvolen overlay v CentralPanel po editor renderingu.

## Known Limitations

- Timestamp je v UTC, ne v lokálním čase — pro přidání lokálního času by byla potřeba chrono nebo libc timezone.
- History panel je jednoduchý list + preview (ne split view s diff) — split view s diff zvýrazněním je scope S02.
- Manuální ověření (save 3× → snapshoty, context menu → panel) neprovedeno v automatizovaném kontextu — pokryto UAT scénáři.

## Follow-ups

- S02 postaví split view s diff zvýrazněním nad `HistoryViewState` a `get_snapshot_content()` z S01.
- S03 přidá cleanup s max_age a edge case handling.

## Files Created/Modified

- `src/app/local_history.rs` — změněná signature take_snapshot() na Result, get_snapshot_content(), 6 unit testů
- `src/app/ui/workspace/state/mod.rs` — nové pole background_io_tx a history_view v WorkspaceState
- `src/app/ui/workspace/state/init.rs` — vytvoření mpsc kanálu, inicializace history_view
- `src/app/ui/workspace/mod.rs` — mod history, send_snapshot_signal() helper, ShowHistory handling, history panel rendering
- `src/app/ui/background.rs` — autosave hook, error handling v LocalHistory handleru s toast
- `src/app/ui/workspace/history/mod.rs` — nový soubor: HistoryViewState, render_history_panel(), format_timestamp(), days_to_date()
- `src/app/ui/widgets/tab_bar.rs` — ShowHistory(usize) varianta v TabBarAction
- `src/app/ui/editor/render/tabs.rs` — context menu na tab, rozšířená signatura tab_bar() o i18n
- `src/app/ui/editor/ui.rs` — předání i18n do tab_bar(), propagace ShowHistory
- `src/app/ui/terminal/right/mod.rs` — match pro ShowHistory v apply_tab_action (no-op)
- `src/app/mod.rs` — dummy background_io_tx a history_view v testovacích konstruktorech
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 6 nových i18n klíčů (tab-context-history, tab-context-close, history-panel-*)
- `locales/{cs,en,sk,de,ru}/errors.ftl` — nový klíč error-history-snapshot

## Forward Intelligence

### What the next slice should know
- `HistoryViewState` je v `src/app/ui/workspace/history/mod.rs` — S02 jej rozšíří o diff state a split view mode.
- `get_snapshot_content()` vrací `io::Result<String>` — S02 potřebuje obsah aktuální verze (z editoru) i historické verze (z této metody) pro diff.
- Context menu trigger je `TabBarAction::ShowHistory(usize)` — S02 může hook do stejného vstupního bodu a přepnout do split view místo simple panelu.
- `render_history_panel()` přijímá split reference kvůli borrow checker — S02 bude potřebovat stejný pattern pro split view render.

### What's fragile
- **Borrow checker kolem WorkspaceState** — inline snapshot hook v autosave/unsaved-close-guard je důsledek borrow konfliktů. Jakákoliv změna workspace state structu vyžaduje ověření, že tyto hooky stále kompilují.
- **History panel overlay positioning** — renderuje se po editoru v CentralPanel. S02 bude muset tento pattern nahradit split view renderem, ne přidávat další vrstvu.

### Authoritative diagnostics
- `ls .polycredo/history/*/` — ověření snapshot souborů na FS po save
- `cargo test -- local_history` — 6 testů pokrývajících pipeline integrity
- Toast v UI — propaguje I/O chyby s cestou souboru a error message

### What assumptions changed
- **Původní předpoklad: sdílený helper pro všechny save hooky** — Borrow checker vyžaduje inline implementaci v autosave a unsaved-close-guard. Helper `send_snapshot_signal()` funguje jen pro manual save.
- **Původní předpoklad: background_io_rx je vždy None** — Kanál je nyní funkční, ale receiver se stále unwrapuje v process_background_events. Kód je robustní díky `if let Some(rx)` patternu.
