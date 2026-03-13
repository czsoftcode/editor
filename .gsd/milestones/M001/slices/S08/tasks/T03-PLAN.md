# T03: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 03

**Slice:** S08 — **Milestone:** M001

## Description

Zajistit staged/sync UX při okamžitém přepnutí sandbox režimu: blokovat OFF při staged souborech a při ON nabídnout automatický sync do sandboxu.

## Must-Haves

- [ ] "OFF režim je blokován při staged souborech a uživatel dostane dialog k vyřešení."
- [ ] "Sandbox staged bar zůstává viditelná i v OFF, dokud není staged vyřešen."
- [ ] "Při ON se nabídne automatický sync projektu do sandboxu a při potvrzení se provede."

## Files

- `src/app/ui/workspace/mod.rs`
- `src/app/sandbox.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
