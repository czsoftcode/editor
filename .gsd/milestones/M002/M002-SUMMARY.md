---
id: M002
provides:
  - Automatický snapshot souboru při každém uložení (manual save, autosave, unsaved-close-guard) pro ne-binární taby
  - Background IO kanál (mpsc::channel) propojující save hooky s LocalHistory::take_snapshot()
  - Tab context menu (pravý klik → "Historie souboru" / "Zavřít tab") s guardem na binární soubory
  - Split view se dvěma read-only ScrollArea panely (aktuální vlevo, historická vpravo) s resize handle
  - Diff zvýraznění přes similar::TextDiff::from_lines() s per-index cachováním (ne per-frame)
  - Navigační šipky (starší/novější) s disabled stavem na hranicích seznamu verzí
  - Retence snapshotu (max 50 verzí, max 30 dní) v background threadu při startu workspace
  - I/O error propagace z background snapshot handleru do UI toastu
  - i18n pokrytí (12 klíčů × 5 jazyků: cs, en, sk, de, ru)
  - 13 unit testů pokrývajících snapshot pipeline, cleanup retenci, diff logiku a barvy
key_decisions:
  - "Save hook signál vrací z save() pro odeslání z workspace kódu, ne přímý sender v Editor structu — nemíchat I/O concerns do editoru"
  - "History split view nahradí normální rendering kompletně (ne overlay) — jednodušší než koexistence s markdown split"
  - "Nezávislý scroll dvou history panelů — sync scroll by vyžadoval diff-aware mapování řádků"
  - "Diff výsledek cachovat per history entry (diff_for_index), ne počítat per-frame — similar::TextDiff je O(n*d)"
  - "cleanup_history_dir() jako standalone funkce bez &self — LocalHistory není Send, pro thread::spawn nutné extrahovat"
  - "take_snapshot() mění signature z Option na Result — nutné pro S-3 (I/O error propagace)"
  - "Timestamp formátování bez chrono dependency — vlastní days_to_date() (Howard Hinnant) v UTC"
  - "Inline snapshot hook v autosave/unsaved-close-guard kvůli borrow checker konfliktu na pending_close_flow"
  - "Cleanup I/O chyby jako let _ — background operace při startu, toastování by bylo rušivé"
patterns_established:
  - Snapshot signál po úspěšném save s guardem na is_binary a strip_prefix pro relativní cestu
  - take_snapshot() vrací Result pro propagaci I/O chyb místo tichého swallowingu
  - DiffLine struct s owned String pro cachování diff výsledků přes framy
  - DiffColors s dark/light větvením — znovupoužitelný vzor pro budoucí diff rendering
  - Podmíněný editor rendering v history/split mode (editor.ui() se nevolá v history mode)
  - Standalone FS utility funkce pro background thread operace (cleanup_history_dir pattern)
  - Tab context menu přes r.context_menu() s i18n texty a guardem
observability_surfaces:
  - Toast při I/O chybě snapshotování (i18n klíč error-history-snapshot s parametry path a reason)
  - Toast "Žádné historické verze" při pokusu o otevření historie pro soubor bez snapshotu
  - Snapshot soubory na FS v .polycredo/history/ jako inspectable artifact
  - Chybová hláška v pravém diff panelu při selhání čtení snapshot souboru z disku
  - HistoryViewState.diff_for_index indikuje diff cache hit/miss
requirement_outcomes:
  - id: S-3
    from_status: active
    to_status: active
    proof: "M002 implementuje S-3 částečně — I/O chyby ze snapshot pipeline se propagují do toastu (error-history-snapshot klíč ve všech 5 jazycích, handler v background.rs). Zbývají další I/O operace mimo history scope (file save, watcher, aj.), proto S-3 zůstává active."
duration: ~90 min (S01 50min + S02 25min + S03 15min)
verification_result: passed
completed_at: 2026-03-13
---

# M002: Local History

**Automatický snapshot při uložení souboru s plnohodnotným split view pro diff prohlížení historie — od save hooku přes background pipeline až po dual-pane UI s navigací a retencí.**

## What Happened

Milestone M002 provedl tři slice ve vzestupném pořadí rizika:

**S01 (Snapshot Pipeline + Tab Context Menu)** oživil mrtvý background IO kanál — vytvořil `mpsc::channel()` v `init_workspace`, propojil tři save hooky (manual save, autosave, unsaved-close-guard) s `FsChangeResult::LocalHistory` signálem. `take_snapshot()` změnil signaturu na `Result<Option<PathBuf>, io::Error>` pro propagaci I/O chyb do toastu. Přidáno context menu na tab bar (pravý klik → "Historie souboru" / "Zavřít tab") a jednoduchý history panel se seznamem verzí a textovým náhledem. 6 unit testů + i18n.

**S02 (Split View s Diff a Navigací)** nahradil jednoduchý panel plnohodnotným split view. Dva read-only ScrollArea panely (aktuální verze vlevo, historická vpravo) s resize handle převzatým z `render/markdown.rs`. Diff engine (`similar::TextDiff::from_lines()`) s cachovaným výsledkem per `selected_index` — přepočet jen při navigaci, ne per-frame. Toolbar s navigačními šipkami (starší/novější), info o verzi a zavíracím tlačítkem. DiffColors s automatickým dark/light větvením. 5 unit testů.

**S03 (Cleanup + Edge Cases)** rozšířil `cleanup()` o `max_age_secs` parametr a extrahoval logiku do standalone `cleanup_history_dir()` funkce spustitelné z background threadu (LocalHistory není Send). Při startu workspace se spustí cleanup s konfigurací 50 verzí / 30 dní. Ošetřeno zavření tabu v history mode (clean i dirty close path). Finální i18n audit potvrdil kompletní pokrytí 12 klíčů × 5 jazyků. 2 nové unit testy.

Celkový výsledek: 13 nových unit testů, 148 celkových testů zelených (1 pre-existující selhání `phase35_delete_foundation` mimo scope).

## Cross-Slice Verification

**Kritérium 1: Po uložení souboru 3× s různým obsahem se v `.polycredo/history/` vytvoří 3 snapshoty.**
- ✅ Tři save hooky (manual, autosave, unsaved-close-guard) odesílají `FsChangeResult::LocalHistory` po úspěšném uložení. Guard `!tab.is_binary` zajišťuje přeskočení binárních tabů. Unit test `take_snapshot_creates_file_on_fs` ověřuje vytvoření souboru na FS. Test `duplicate_content_is_skipped` ověřuje, že identický obsah nevytvoří duplicitní snapshot.

**Kritérium 2: Pravý klik na tab → "Historie souboru" otevře split view se dvěma panely.**
- ✅ Context menu přes `r.context_menu()` v `tabs.rs` s `TabBarAction::ShowHistory(usize)`. Handler v `workspace/mod.rs` inicializuje `HistoryViewState` s `current_content` z aktivního tabu a volá `render_history_split_view()` — dva ScrollArea panely s resize handle.

**Kritérium 3: Šipky přepínají mezi historickými verzemi, diff zvýraznění se aktualizuje.**
- ✅ Navigační šipky v toolbar (starší/novější) s disabled stavem na hranicích. `compute_diff()` produkuje `Vec<DiffLine>` cachovaný per `selected_index` — invalidace přes `diff_for_index != selected_index`. Unit testy `compute_diff_detects_insertions_and_deletions` a `compute_diff_identical_texts_all_equal`.

**Kritérium 4: Zavření history view vrátí editor do normálního režimu.**
- ✅ Zavírací tlačítko (✕) v toolbar nastaví `history_view = None`. Edge case handling v S03: zavření tabu v history mode (clean close i dirty close path) invaliduje `history_view`. `editor.ui()` podmíněné na `history_view.is_none()`.

**Kritérium 5: Soubor s 60 verzemi → po cleanup max 50; verze starší 30 dní smazány.**
- ✅ `cleanup_history_dir()` s `max_versions: 50` a `max_age_secs: Some(30 * 24 * 3600)`. Background thread při startu workspace. Unit testy `cleanup_removes_old_versions_by_age` a `cleanup_respects_max_versions_before_age`.

**Kritérium 6: Binární soubory nespouští snapshot.**
- ✅ Guard `!tab.is_binary` na třech místech: `send_snapshot_signal()` (manual save), autosave hook v `background.rs`, unsaved-close-guard hook.

**Kritérium 7: I/O chyby při snapshotování se propagují do UI toastu.**
- ✅ `take_snapshot()` vrací `Result<Option<PathBuf>, io::Error>`. Background handler v `background.rs` matchuje `Err(e)` a zobrazí toast s i18n klíčem `error-history-snapshot` (path + reason). Unit test `error_on_readonly_directory` ověřuje error path.

**Průřezová verifikace:**
- `cargo check` — bez chyb
- `cargo clippy` — bez warningů
- `cargo test` — 148 testů prošlo, 1 pre-existující selhání mimo scope
- i18n grep audit — 12 klíčů kompletních ve všech 5 jazycích (cs, en, sk, de, ru)

**Manuální UAT:**
- Vyžaduje GUI desktop prostředí. Scénáře popsány v S01-UAT.md a S02-UAT.md. Headless verifikace pokrývá kompilaci, unit testy a logiku; vizuální layout a UX vyžaduje spuštění editoru.

## Requirement Changes

- **S-3** (Neignorovat I/O chyby, propagovat do UI toastu): active → active — M002 implementuje S-3 pro history pipeline (I/O chyby ze snapshot operací se propagují do toastu s error-history-snapshot klíčem). Zbývající I/O operace (file save, watcher, general FS operace) jsou mimo scope M002. Požadavek zůstává active s částečným pokrytím.

## Forward Intelligence

### What the next milestone should know
- Background IO kanál (`background_io_tx`/`background_io_rx`) je nyní funkční — lze ho rozšířit o další `FsChangeResult` varianty pro budoucí background operace.
- `HistoryViewState` je kompletní a rozšiřitelný — pro budoucí "restore verze" feature stačí přidat handler na vybranou verzi.
- Borrow checker kolem `WorkspaceState` je hlavní omezení — inline snapshot hooky v autosave/unsaved-close-guard jsou důsledek. Jakákoliv změna workspace state structu vyžaduje ověření kompilace těchto hooků.

### What's fragile
- **Borrow checker v save hoocích** — `send_snapshot_signal()` funguje jen pro manual save. Autosave a unsaved-close-guard mají inline implementaci kvůli borrow konfliktu na `pending_close_flow`. Restrukturalizace `WorkspaceState` může tyto hooky rozbít.
- **Snapshot timestamp parsing v cleanup** — `cleanup_history_dir()` parsuje timestamp z názvu souboru (`_` split, pozice 0). Změna formátu pojmenování snapshotů rozbije cleanup.
- **current_content v history view** — načte se jednou při otevření. Pokud se soubor změní externě během prohlížení historie, obsah neodpovídá aktuálnímu stavu na disku.

### Authoritative diagnostics
- `ls -la .polycredo/history/*/` — přímá inspekce snapshot retence a existence na disku
- `cargo test -- local_history` — 8 testů pokrývajících pipeline integrity, duplikáty, cleanup, error path
- `cargo test -- history::tests` — 5 testů pro diff logiku, barvy, timestamp formátování
- Toast v UI — propaguje I/O chyby s cestou souboru a error message

### What assumptions changed
- **Sdílený helper pro všechny save hooky** → Borrow checker vyžaduje inline implementaci v autosave/unsaved-close-guard. Helper jen pro manual save.
- **background_io_rx vždy None** → Kanál je nyní funkční, receiver se unwrapuje přes `if let Some(rx)` pattern.
- **Odhadovaná doba** — S02 odhadováno na 90 min, realizováno za ~25 min díky dobré přípravě z S01 a existujícím vzorům v codebase.
- **history-panel-no-workspace klíč** → nepoužívá se v kódu, audit zjistil 10 reálně používaných klíčů (ne 11).

## Files Created/Modified

- `src/app/local_history.rs` — rozšířen take_snapshot() na Result, get_snapshot_content(), cleanup_history_dir() standalone, 8 unit testů
- `src/app/ui/workspace/history/mod.rs` — nový modul: HistoryViewState, DiffLine, DiffColors, compute_diff(), render_history_split_view(), 5 unit testů
- `src/app/ui/workspace/state/mod.rs` — nová pole background_io_tx a history_view ve WorkspaceState
- `src/app/ui/workspace/state/init.rs` — mpsc kanál, background cleanup thread
- `src/app/ui/workspace/mod.rs` — send_snapshot_signal(), ShowHistory handling, podmíněný editor rendering, history_view invalidace
- `src/app/ui/background.rs` — autosave hook, unsaved-close-guard hook, LocalHistory error handling s toast
- `src/app/ui/widgets/tab_bar.rs` — ShowHistory(usize) varianta v TabBarAction
- `src/app/ui/editor/render/tabs.rs` — context menu na tab, rozšířená signatura tab_bar() o i18n
- `src/app/ui/editor/ui.rs` — předání i18n do tab_bar(), propagace ShowHistory
- `src/app/ui/terminal/right/mod.rs` — match pro ShowHistory v apply_tab_action
- `src/app/mod.rs` — dummy background_io_tx a history_view v testovacích konstruktorech
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 11 nových i18n klíčů (tab-context-*, history-panel-*, history-nav-*, history-*-label, history-version-info)
- `locales/{cs,en,sk,de,ru}/errors.ftl` — nový klíč error-history-snapshot
