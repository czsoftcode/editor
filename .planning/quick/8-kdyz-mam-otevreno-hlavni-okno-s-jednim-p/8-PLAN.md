# Quick Plan: Oprava křížku v hlavním okně

## Cíl
Zajistit, aby kliknutí na křížek v hlavním okně při otevřeném projektu zavřelo aktuální projekt (návrat na startup), stejně jako zavírání projektu v sekundárních oknech.

## Tasky

1. Přesměrovat root close flow na zavření projektu
- V `src/app/mod.rs` upravit obsluhu `viewport().close_requested()` tak, aby při `root_ws.is_some()` používala `GlobalCloseKind::RootProjectClose` místo `RootViewportClose`.
- Zachovat existující unsaved guard flow a potvrzovací modal.

2. Doplnit regresní test pro root project close bez dirty tabů
- Přidat unit test do `src/app/mod.rs`, který ověří, že `start_global_close_guard(GlobalCloseKind::RootProjectClose, ...)` bez neuložených změn otevře `show_close_project_confirm` a nespustí quit flow.

3. Ověřit build/test gate
- Spustit `cargo check` a `./check.sh`.
- Zapsat výsledek do quick summary.
