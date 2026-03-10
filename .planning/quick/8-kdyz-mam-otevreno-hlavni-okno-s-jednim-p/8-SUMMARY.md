# Quick Task 8 Summary

## Cíl
Zprovoznit křížek v hlavním okně tak, aby při otevřeném projektu zavřel aktuální projekt (návrat na startup), neblokoval se a nechoval se odlišně od close flow projektu.

## Co bylo provedeno

### Task 1: Root close flow přesměrován na zavření projektu
- V `src/app/mod.rs` je v obsluze `viewport().close_requested()` při `root_ws.is_some()` změna:
  - `GlobalCloseKind::RootViewportClose` -> `GlobalCloseKind::RootProjectClose`
- Tím křížek v hlavním okně spouští stejný close-project flow (včetně unsaved guard/confirm), místo pokusu zavřít celý root viewport.

### Task 2: Regresní test
- Přidán test `root_project_close_without_dirty_tabs_opens_close_project_confirm`.
- Test ověřuje, že `start_global_close_guard(GlobalCloseKind::RootProjectClose, ..)` bez dirty tabů:
  - nastaví `show_close_project_confirm = true`
  - nespustí quit flow (`show_quit_confirm = false`)
  - nenechá viset `pending_global_close`.

## Validace
- `cargo check`: **OK**
- `cargo test root_project_close_without_dirty_tabs_opens_close_project_confirm --bin polycredo-editor`: **OK**
- `./check.sh`: **FAIL (unrelated)**
  - Fail je na `cargo fmt --check` kvůli již existujícím neformátovaným souborům mimo scope této opravy.

## Commity
- Neprovedeno (změny ponechány ve working tree).
