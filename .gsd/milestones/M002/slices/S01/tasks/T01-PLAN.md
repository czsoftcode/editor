---
estimated_steps: 8
estimated_files: 6
---

# T01: Oživit background IO kanál a napojit save hook na snapshot pipeline

**Slice:** S01 — Snapshot Pipeline a Tab Context Menu
**Milestone:** M002

## Description

Celý background IO kanál (`background_io_tx/rx`) je mrtvý kód — receiver je vždy `None`, sender nikde nevzniká. Tento task oživí kanál, napojí save hooky na snapshot signál a zajistí propagaci I/O chyb do toastu. Přidá `get_snapshot_content()` pro čtení historických verzí a pokryje vše unit testy.

## Steps

1. **Vytvořit kanál v `init_workspace`:** V `src/app/ui/workspace/state/init.rs` vytvořit `mpsc::channel::<FsChangeResult>()`. Sender uložit do nového pole `background_io_tx: mpsc::Sender<FsChangeResult>` ve `WorkspaceState`. Receiver přiřadit do existujícího `background_io_rx: Some(rx)`. Totéž v `src/app/mod.rs` (oba konstrukční body WorkspaceState, řádky ~985 a ~1110).
2. **Save hook v `handle_manual_save_action`:** V `src/app/ui/workspace/mod.rs` — po úspěšném `ws.editor.save()` v `ManualSaveRequest::SaveEditorFile` větvi: pokud aktivní tab není binární, extrahovat relativní cestu (vůči `ws.root_path`) a obsah (`tab.content.clone()`), odeslat přes `ws.background_io_tx.send(FsChangeResult::LocalHistory(rel_path, content))`.
3. **Save hook v autosave:** V `src/app/ui/background.rs` — po úspěšném `try_autosave` (v sekci 5, kde `should_autosave` je true a save proběhne bez chyby): stejný pattern — ne-binární → odeslat snapshot signál. Pozor: `ws.background_io_tx` je dostupný protože `ws` je `&mut WorkspaceState`.
4. **Save hook v unsaved-close-guard:** V `src/app/ui/workspace/mod.rs` — v `UnsavedGuardDecision::Save` větvi (kolem řádku 328): po úspěšném save odeslat snapshot signál se stejným guardem na binární taby.
5. **Změnit `take_snapshot()` signature:** V `src/app/local_history.rs` změnit návratový typ na `Result<Option<PathBuf>, std::io::Error>`. `fs::write` chybu propagovat nahoru. `fs::create_dir_all` chybu taky propagovat. Early returns pro deduplikaci a `.polycredo` filtr zůstávají jako `Ok(None)`.
6. **Error handling v background handleru:** V `src/app/ui/background.rs` v `FsChangeResult::LocalHistory` větvi — pokud `take_snapshot()` vrátí `Err(e)`, vytvořit toast s chybou. Použít i18n klíč `error-history-snapshot` s parametry `path` a `reason`. Přidat tento klíč do všech 5 locale souborů (`locales/*/errors.ftl`).
7. **Implementovat `get_snapshot_content()`:** V `src/app/local_history.rs` přidat `pub fn get_snapshot_content(&self, relative_file_path: &Path, entry: &HistoryEntry) -> io::Result<String>`. Sestavit cestu: `base_dir / encode_path(rel) / {timestamp}_{hash}.txt`, přečíst `fs::read_to_string`.
8. **Unit testy:** V `src/app/local_history.rs` přidat `#[cfg(test)] mod tests` s testy: (a) `take_snapshot` vytvoří soubor na FS v tmpdir, (b) duplikovaný obsah → skip, (c) `.polycredo` cesta → skip (`Ok(None)`), (d) `get_snapshot_content` vrátí správný obsah, (e) `get_history` vrátí seřazené záznamy, (f) error handling — write do neexistujícího readonly adresáře.

## Must-Haves

- [ ] `background_io_tx` sender existuje ve WorkspaceState a je propojený s `background_io_rx` receiverem.
- [ ] Manuální save, autosave i unsaved-close-guard save odesílají `FsChangeResult::LocalHistory` pro ne-binární taby.
- [ ] `take_snapshot()` vrací `Result<Option<PathBuf>, io::Error>` — I/O chyby se nepotlačují.
- [ ] Background handler zobrazí toast při snapshot I/O chybě.
- [ ] `get_snapshot_content()` načte obsah historické verze z disku.
- [ ] i18n klíč `error-history-snapshot` ve všech 5 locale souborech.
- [ ] Unit testy prochází — pokrývají snapshot, deduplikaci, binární skip, get_snapshot_content.

## Verification

- `cargo test --lib -- local_history` — všechny testy projdou.
- `cargo check` — kompilace bez chyb (ověří napojení senderu ve všech save hookách).
- `./check.sh` — celkový projekt check.

## Observability Impact

- Signals added/changed: Toast při I/O chybě snapshotování (nový i18n klíč `error-history-snapshot`). `take_snapshot()` mění signature z `Option` na `Result` — volající musí handlovat chybu.
- How a future agent inspects this: `ls .polycredo/history/*/` pro snapshot soubory. Toast viditelný v UI.
- Failure state exposed: I/O error message + cesta souboru v toastu.

## Inputs

- `src/app/local_history.rs` — existující `LocalHistory` struct s `take_snapshot()`, `get_history()`, `cleanup()`.
- `src/app/ui/workspace/state/mod.rs` — `WorkspaceState` s `background_io_rx: Option<Receiver>` a `local_history`.
- `src/app/ui/workspace/state/init.rs` — konstrukční logika workspace.
- `src/app/ui/workspace/mod.rs` — `handle_manual_save_action`, unsaved close guard flow.
- `src/app/ui/background.rs` — `process_background_events` s `FsChangeResult::LocalHistory` handlerem.
- DECISIONS.md: "Save hook signál vrací z save() pro odeslání z workspace kódu, ne přímý sender v Editor structu".

## Expected Output

- `src/app/local_history.rs` — rozšířený o `get_snapshot_content()`, změněná signature `take_snapshot()`, unit testy.
- `src/app/ui/workspace/state/mod.rs` — nové pole `background_io_tx: mpsc::Sender<FsChangeResult>`.
- `src/app/ui/workspace/state/init.rs` — vytvoření kanálu a přiřazení obou konců.
- `src/app/mod.rs` — totéž v obou konstrukčních bodech.
- `src/app/ui/workspace/mod.rs` — save hooky v manual save a unsaved-close-guard.
- `src/app/ui/background.rs` — autosave hook + error handling v LocalHistory handleru.
- `locales/*/errors.ftl` — nový klíč `error-history-snapshot`.
