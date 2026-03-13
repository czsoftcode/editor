---
id: T01
parent: S01
milestone: M002
provides:
  - Funkční background IO kanál (background_io_tx/rx propojený přes mpsc::channel)
  - Save hooky (manual, autosave, unsaved-close-guard) odesílají FsChangeResult::LocalHistory pro ne-binární taby
  - take_snapshot() vrací Result<Option<PathBuf>, io::Error> — I/O chyby se propagují
  - get_snapshot_content() pro čtení historických verzí z disku
  - Toast error handling při snapshot I/O chybě
  - i18n klíč error-history-snapshot ve všech 5 locale souborech
  - 6 unit testů pokrývajících snapshot pipeline
key_files:
  - src/app/local_history.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/background.rs
  - src/app/mod.rs
  - locales/cs/errors.ftl
  - locales/en/errors.ftl
  - locales/de/errors.ftl
  - locales/ru/errors.ftl
  - locales/sk/errors.ftl
key_decisions:
  - Helper funkce send_snapshot_signal() extrahována v workspace/mod.rs pro manual save a obecné použití; v unsaved-close-guard a autosave se signál odesílá inline kvůli borrow checker konfliktům (flow drží &mut ws.pending_close_flow)
patterns_established:
  - Snapshot signál se odesílá po úspěšném save s guardem na is_binary a strip_prefix pro relativní cestu
  - take_snapshot() vrací Result pro propagaci I/O chyb místo tichého swallowingu
observability_surfaces:
  - Toast při I/O chybě snapshotování (i18n klíč error-history-snapshot s parametry path a reason)
  - Snapshot soubory na FS v .polycredo/history/ jako inspectable artifact
  - ls .polycredo/history/*/ pro ověření počtu snapshotů
duration: 30min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Oživit background IO kanál a napojit save hook na snapshot pipeline

**Oživený background IO kanál s propojením save hooků na snapshot pipeline, propagací I/O chyb do toastu a get_snapshot_content() pro čtení historických verzí.**

## What Happened

1. **Kanál v init_workspace** — vytvořen `mpsc::channel::<FsChangeResult>()` v `init.rs`. Sender uložen do nového pole `background_io_tx` ve `WorkspaceState`, receiver přiřazen do `background_io_rx: Some(rx)`. Oba testové konstrukční body v `app/mod.rs` dostaly dummy sender.

2. **Save hooky** — po úspěšném save se pro ne-binární taby odesílá `FsChangeResult::LocalHistory(rel_path, content)`:
   - Manual save v `handle_manual_save_action` — přes helper `send_snapshot_signal()`
   - Autosave v `process_background_events` — inline kvůli borrow checker
   - Unsaved-close-guard save v `process_unsaved_close_guard_dialog` — inline kvůli borrow na `flow`

3. **take_snapshot() signature** — změněna z `Option<PathBuf>` na `Result<Option<PathBuf>, io::Error>`. `fs::write` a `fs::create_dir_all` chyby se propagují. Early returns pro deduplikaci a `.polycredo` filtr vrací `Ok(None)`.

4. **Error handling** — background handler v `FsChangeResult::LocalHistory` větvi handluje `Err` z `take_snapshot()` a zobrazí toast s i18n klíčem `error-history-snapshot`.

5. **get_snapshot_content()** — nová metoda v `LocalHistory` sestaví cestu z `base_dir / encode_path(rel) / {timestamp}_{hash}.txt` a přečte `fs::read_to_string`.

6. **i18n** — klíč `error-history-snapshot` přidán do všech 5 locale souborů (cs, en, de, ru, sk).

7. **Unit testy** — 6 testů: vytvoření snapshotu, deduplikace, .polycredo skip, get_snapshot_content, seřazené záznamy, error na readonly adresáři.

## Verification

- `cargo test -- local_history` — **6/6 testů prošlo** (take_snapshot_creates_file_on_fs, duplicate_content_is_skipped, polycredo_path_is_skipped, get_snapshot_content_returns_correct_data, get_history_returns_sorted_entries, error_on_readonly_directory)
- `cargo check` — **kompilace bez chyb**
- `cargo clippy` — **žádné warningy**
- `./check.sh` — fmt a clippy prochází, kompilace OK. Jeden pre-existující failing test (`phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols`) nesouvisí s touto prací — hledá chybějící soubor `.planning/phases/35-trash-foundation-async-safety/35-03-PLAN.md`.

### Slice-level verification status (T01 = intermediate task):
- ✅ `cargo test -- local_history` — prochází
- ✅ `cargo check` — prochází
- ⚠️ `./check.sh` — prochází s výjimkou pre-existujícího failing testu
- ⏳ Manuální ověření (save 3× → snapshoty, context menu, history panel) — čeká na T02

## Diagnostics

- `ls .polycredo/history/*/` — ověření snapshot souborů na FS
- `cat .polycredo/history/index.json` — mapování hash→cesta
- Toast v UI při I/O chybě — obsahuje cestu souboru a chybovou hlášku
- `take_snapshot()` vrací `Result<Option<PathBuf>, io::Error>` — volající musí handlovat chybu

## Deviations

- V unsaved-close-guard a autosave hook se snapshot signál odesílá inline místo přes helper `send_snapshot_signal()` — důvod: borrow checker neumožňuje předat `&ws` když `flow` drží `&mut ws.pending_close_flow`. Funkčně ekvivalentní.

## Known Issues

- Pre-existující failing test `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` — chybějící soubor `.planning/phases/35-...`. Nesouvisí s touto prací.

## Files Created/Modified

- `src/app/local_history.rs` — změněná signature `take_snapshot()` na Result, přidaný `get_snapshot_content()`, přidaný import `io`, přidaný test modul s 6 testy
- `src/app/ui/workspace/state/mod.rs` — nové pole `background_io_tx: mpsc::Sender<FsChangeResult>` ve WorkspaceState
- `src/app/ui/workspace/state/init.rs` — vytvoření mpsc kanálu, přiřazení sender/receiver
- `src/app/ui/workspace/mod.rs` — helper `send_snapshot_signal()`, save hook v manual save, save hook v unsaved-close-guard
- `src/app/ui/background.rs` — autosave hook, error handling v LocalHistory handleru s toast
- `src/app/mod.rs` — oba testové konstrukční body WorkspaceState dostaly dummy `background_io_tx`
- `locales/cs/errors.ftl` — nový klíč `error-history-snapshot`
- `locales/en/errors.ftl` — nový klíč `error-history-snapshot`
- `locales/de/errors.ftl` — nový klíč `error-history-snapshot`
- `locales/ru/errors.ftl` — nový klíč `error-history-snapshot`
- `locales/sk/errors.ftl` — nový klíč `error-history-snapshot`
