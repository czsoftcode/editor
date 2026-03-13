# S08: Okam It Aplikov N Zm Ny Re Imu Sandboxu Po P Epnut Checkboxu

**Goal:** Zavést okamžité apply sandbox režimu po Save v Settings včetně potvrzení při OFF, správného pořadí persist → runtime apply, a bezpečné propagace do všech oken stejného projektu s korektními toasty a možností odložení.
**Demo:** Zavést okamžité apply sandbox režimu po Save v Settings včetně potvrzení při OFF, správného pořadí persist → runtime apply, a bezpečné propagace do všech oken stejného projektu s korektními toasty a možností odložení.

## Must-Haves


## Tasks

- [x] **T01: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 01** `est:29min`
  - Zavést okamžité apply sandbox režimu po Save v Settings včetně potvrzení při OFF, správného pořadí persist → runtime apply, a bezpečné propagace do všech oken stejného projektu s korektními toasty a možností odložení.
- [x] **T02: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 02** `est:17m`
  - Zajistit, že runtime přepnutí sandbox režimu správně restartuje terminály, přemapuje file tree a otevřené taby, blokuje OFF při staged změnách a nabízí sync při ON.
- [x] **T03: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 03** `est:15min`
  - Zajistit staged/sync UX při okamžitém přepnutí sandbox režimu: blokovat OFF při staged souborech a při ON nabídnout automatický sync do sandboxu.
- [x] **T04: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 04** `est:5min`
  - Uzavřít dvě SANDBOX-03 mezery nalezené verifikací fáze 05.

Gap 1: `pending_tab_remap` zůstával nastaven donekonečna po vypršení toastu se SandboxRemapTabs akcí — cleanup nikdy neproběhl.
Gap 2: Verifikátor označil label-timing jako "nezaručenou podmínku"; dokumentační komentář v kódu tento záměr ujasní.

Purpose: Plné splnění SANDBOX-03 (runtime apply restartuje terminály, přepíná file tree root a přemapovává taby).
Output: Cleanup logika v panels.rs + dokumentační komentář v state/mod.rs.

## Files Likely Touched

- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/mod.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/workspace/modal_dialogs/conflict.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `src/app/ui/terminal/mod.rs`
- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/terminal/bottom/build_bar.rs`
- `src/app/ui/panels.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/editor/files.rs`
- `src/app/ui/editor/ui.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `src/app/ui/workspace/mod.rs`
- `src/app/sandbox.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `src/app/ui/panels.rs`
- `src/app/ui/workspace/state/mod.rs`
