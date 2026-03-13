# S01: Snapshot Pipeline a Tab Context Menu

**Goal:** Po uložení souboru se automaticky vytvoří snapshot v `.polycredo/history/`. Pravý klik na tab ukáže context menu s "Historie souboru", které otevře jednoduchý history panel s výpisem verzí a náhledem vybrané verze. Binární taby jsou přeskočeny, I/O chyby propagovány do toastu.
**Demo:** Uložení souboru 3× s různým obsahem → 3 snapshoty na FS. Pravý klik na tab → "Historie souboru" → panel s verzemi + náhled textu vybrané verze. Binární tab nemá "Historie souboru" v menu. I/O chyba při snapshotování → toast.

## Must-Haves

- `background_io_tx` sender existuje a je propojený do save hooku — po úspěšném uložení se odešle `FsChangeResult::LocalHistory`.
- Save hook pokrývá manuální save (`handle_manual_save_action`), autosave (`try_autosave`) i unsaved-close-guard save.
- Binární taby neposílají snapshot signál.
- `take_snapshot()` propaguje I/O chyby — `background.rs` handler je zobrazí v toastu.
- `LocalHistory::get_snapshot_content()` načte obsah historické verze z disku.
- Pravý klik na tab v tab baru zobrazí context menu s položkou "Historie souboru" (a "Zavřít tab").
- "Historie souboru" otevře history panel — výpis verzí seřazený od nejnovější, klik na verzi zobrazí náhled textu.
- `HistoryViewState` struct nese stav panelu (soubor, vybraná verze, seznam verzí, obsah náhledu).
- i18n klíče pro context menu a history panel ve všech 5 jazycích (cs, en, sk, de, ru).
- `cargo check` + `./check.sh` prochází.

## Proof Level

- This slice proves: integration
- Real runtime required: yes (manuální ověření v běžícím editoru — save → snapshot, context menu → panel)
- Human/UAT required: yes (vizuální kontrola context menu a history panelu)

## Verification

- `cargo test --lib -- local_history` — unit testy pro snapshot pipeline, binární detekci, get_snapshot_content, I/O error propagaci.
- `cargo check` — kompilace bez chyb.
- `./check.sh` — celkový projekt check.
- Manuální ověření: spustit editor, uložit soubor 3×, ověřit 3 snapshoty v `.polycredo/history/`. Pravý klik na tab → "Historie souboru" → panel s verzemi. Klik na verzi → náhled textu.

## Observability / Diagnostics

- Runtime signals: Toast při I/O chybě snapshotování. Snapshot soubory na FS v `.polycredo/history/` jako inspectable artifact.
- Inspection surfaces: `ls .polycredo/history/*/` pro ověření počtu snapshotu. `cat .polycredo/history/index.json` pro mapování cest.
- Failure visibility: Toast message obsahuje chybu a cestu souboru. `take_snapshot()` vrací `Result<Option<PathBuf>, io::Error>`.

## Integration Closure

- Upstream surfaces consumed: `Editor::save()` return value (None = success), `WorkspaceState::local_history`, `WorkspaceState::background_io_rx`, `FsChangeResult::LocalHistory` variant.
- New wiring introduced in this slice: `background_io_tx` Sender propojený přes `mpsc::channel()` do `WorkspaceState`. Save hooky ve workspace odesílají snapshot signál. Tab bar reaguje na right-click. `HistoryViewState` v `WorkspaceState`.
- What remains before the milestone is truly usable end-to-end: S02 (split view + diff rendering + navigace šipkami), S03 (cleanup max_age + edge cases).

## Tasks

- [x] **T01: Oživit background IO kanál a napojit save hook na snapshot pipeline** `est:2h`
  - Why: Celý snapshot pipeline je mrtvý — `background_io_rx` je vždy `None`, nikde se nevytváří sender. Bez funkčního kanálu se snapshoty nevytvářejí. Toto je základ celé slice.
  - Files: `src/app/ui/workspace/state/mod.rs`, `src/app/ui/workspace/state/init.rs`, `src/app/ui/workspace/mod.rs`, `src/app/ui/background.rs`, `src/app/local_history.rs`, `src/app/mod.rs`
  - Do: (1) Vytvořit `mpsc::channel()` v `init_workspace` a uložit sender do `WorkspaceState` (nové pole `background_io_tx`), receiver do `background_io_rx`. (2) Po úspěšném `editor.save()` v `handle_manual_save_action` extrahovat relativní cestu a obsah aktivního tabu a odeslat `FsChangeResult::LocalHistory` přes sender — ale jen pro ne-binární taby. (3) Stejný hook přidat do autosave flow v `process_background_events` (po úspěšném `try_autosave`). (4) Pokrýt unsaved-close-guard save flow. (5) Změnit `take_snapshot()` signature na `Result<Option<PathBuf>, std::io::Error>` — propagovat fs::write chybu místo tichého swallowingu. (6) V `background.rs` handleru `FsChangeResult::LocalHistory` — pokud `take_snapshot` vrátí `Err`, odeslat toast s chybou a cestou souboru. (7) Přidat `get_snapshot_content(rel_path, entry) -> io::Result<String>` metodu do `LocalHistory`. (8) Unit testy: snapshot vytvoření, deduplikace, binární skip (`.polycredo` cesta), get_snapshot_content, I/O error na neexistující adresář.
  - Verify: `cargo test --lib -- local_history` projde. `cargo check` projde. `./check.sh` projde.
  - Done when: Po úspěšném uložení ne-binárního souboru se snapshot objeví v `.polycredo/history/`. I/O chyby se propagují. `get_snapshot_content()` vrací obsah. Unit testy prochází.

- [x] **T02: Tab context menu, HistoryViewState a history panel UI s i18n** `est:2h`
  - Why: Uživatel potřebuje způsob jak otevřít historii souboru — context menu na tabu je přirozený vstupní bod. History panel zobrazí seznam verzí a náhled vybrané verze. I18n pokrývá všech 5 jazyků.
  - Files: `src/app/ui/editor/render/tabs.rs`, `src/app/ui/editor/mod.rs`, `src/app/ui/workspace/history/mod.rs`, `src/app/ui/workspace/state/mod.rs`, `src/app/ui/workspace/mod.rs`, `src/app/ui/widgets/tab_bar.rs`, `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl`
  - Do: (1) Přidat `TabBarAction::ShowHistory(usize)` variantu. (2) V `render/tabs.rs` — na `selectable_label` response přidat `.context_menu()` s položkami "Historie souboru" a "Zavřít tab" (přes i18n klíče). (3) Vytvořit `HistoryViewState` struct v `src/app/ui/workspace/history/mod.rs` — drží `file_path`, `relative_path`, `entries: Vec<HistoryEntry>`, `selected_index`, `preview_content: Option<String>`. (4) Přidat `history_view: Option<HistoryViewState>` do `WorkspaceState`. (5) Při `TabBarAction::ShowHistory` — načíst `get_history()` a otevřít panel. (6) Renderovat history panel — pokud `history_view.is_some()`, zobrazit v CentralPanel pod tab barem: seznam verzí vlevo (ScrollArea), náhled textu vpravo (read-only ScrollArea s monospace fontem). Klik na verzi → `get_snapshot_content()` → update preview. Zavírací tlačítko. (7) Přidat i18n klíče: `tab-context-history`, `tab-context-close`, `history-panel-title`, `history-panel-no-versions`, `history-panel-version-label`, `history-panel-close` do všech 5 jazyků. (8) Ověřit, že binární tab nemá "Historie souboru" v context menu.
  - Verify: `cargo check` projde. `./check.sh` projde. Manuální test: pravý klik na tab → context menu viditelné → "Historie souboru" otevře panel → výběr verze ukáže náhled.
  - Done when: Context menu na tabu funguje. History panel zobrazuje verze a náhled. Binární tab nemá "Historie souboru". I18n klíče existují ve všech 5 jazycích. `cargo check` + `./check.sh` prochází.

## Files Likely Touched

- `src/app/local_history.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/background.rs`
- `src/app/ui/editor/render/tabs.rs`
- `src/app/ui/editor/mod.rs`
- `src/app/ui/workspace/history/mod.rs`
- `src/app/ui/widgets/tab_bar.rs`
- `src/app/mod.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `locales/sk/ui.ftl`
- `locales/de/ui.ftl`
- `locales/ru/ui.ftl`
