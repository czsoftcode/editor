# T02: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 02

**Slice:** S08 — **Milestone:** M001

## Description

Zajistit, že runtime přepnutí sandbox režimu správně restartuje terminály, přemapuje file tree a otevřené taby, blokuje OFF při staged změnách a nabízí sync při ON.

## Must-Haves

- [ ] "Při změně sandbox režimu se restartují všechny terminály, ale běžící procesy doběhnou; nové procesy už běží v novém režimu."
- [ ] "File tree se přepne na odpovídající root, otevřené taby se přemapují a neexistující soubory zůstanou otevřené s viditelným stavem."

## Files

- `src/app/ui/terminal/mod.rs`
- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/terminal/bottom/build_bar.rs`
- `src/app/ui/panels.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/editor/files.rs`
- `src/app/ui/editor/ui.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
