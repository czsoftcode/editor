# T04: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 04

**Slice:** S08 — **Milestone:** M001

## Description

Uzavřít dvě SANDBOX-03 mezery nalezené verifikací fáze 05.

Gap 1: `pending_tab_remap` zůstával nastaven donekonečna po vypršení toastu se SandboxRemapTabs akcí — cleanup nikdy neproběhl.
Gap 2: Verifikátor označil label-timing jako "nezaručenou podmínku"; dokumentační komentář v kódu tento záměr ujasní.

Purpose: Plné splnění SANDBOX-03 (runtime apply restartuje terminály, přepíná file tree root a přemapovává taby).
Output: Cleanup logika v panels.rs + dokumentační komentář v state/mod.rs.

## Must-Haves

- [ ] "pending_tab_remap se automaticky vymaže, pokud toast s akcí SandboxRemapTabs vyprší bez interakce uživatele"
- [ ] "apply_sandbox_mode_change() je zdokumentována: label terminálu se mění okamžitě (nový Terminal se novým working_dir), stará instance dobíhá v retired_terminals"

## Files

- `src/app/ui/panels.rs`
- `src/app/ui/workspace/state/mod.rs`
