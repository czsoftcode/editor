---
status: diagnosed
trigger: "Diagnostikuj root cause pro UAT gap: unsaved-guard-esc-focus"
created: 2026-03-10T18:04:15Z
updated: 2026-03-10T18:05:45Z
---

## Current Focus

hypothesis: Potvrzeno: guard modal není v input-gatingu, Esc není explicitně obslouženo v guard modalu a guard flow opakovaně žádá focus editoru.
test: Diagnóza hotová.
expecting: N/A
next_action: vrátit ROOT CAUSE FOUND

## Symptoms

expected: V guard dialogu volba Zrušit (i Esc/zavření dialogu) ponechá kartu otevřenou a ukončí close flow bez vedlejších efektů.
actual: Esc nefunguje - fokus ma editor a ne modal, jinak pass.
errors: None reported
reproduction: Test 5 in UAT
started: Discovered during UAT

## Eliminated

## Evidence

- timestamp: 2026-03-10T18:05:20Z
  checked: src/app/ui/workspace/mod.rs
  found: `dialog_open_base` nezahrnuje `ws.pending_close_flow` ani jiný flag unsaved guard dialogu; editor tak běží, jako by žádný modal nebyl otevřen.
  implication: Editor input/focus logika není guard dialogem deaktivována.

- timestamp: 2026-03-10T18:05:20Z
  checked: src/app/ui/workspace/mod.rs + src/app/ui/editor/tabs.rs
  found: `process_unsaved_close_guard_dialog` volá v každém frame `ws.editor.open_file(&current_path)`, a `open_file` vždy nastaví `focus_editor_requested = true`.
  implication: Guard flow průběžně požaduje focus zpět do editoru místo do modalu.

- timestamp: 2026-03-10T18:05:20Z
  checked: src/app/ui/dialogs/confirm.rs + src/app/ui/widgets/modal.rs
  found: `show_unsaved_close_guard_dialog` spoléhá na `show_flag` a komentář o Esc/X, ale explicitně Esc nečte ani nekonzumuje; `StandardModal::show` také neobsahuje vlastní key handling pro Escape.
  implication: Bez focusu na modalu Esc nevede na cancel guard flow.

- timestamp: 2026-03-10T18:05:45Z
  checked: line-level čtení confirm.rs, modal.rs, workspace/mod.rs, editor/tabs.rs, editor/render/normal.rs
  found: `dialog_open_base` neobsahuje unsaved guard (workspace/mod.rs:420-426), guard flow volá `ws.editor.open_file` (workspace/mod.rs:225), `open_file` nastaví `focus_editor_requested=true` (editor/tabs.rs:10), a editor při `!dialog_open` provede `request_focus(edit_id)` (editor/render/normal.rs:65-68,177-182). Guard dialog ani StandardModal explicitně Escape neřeší (confirm.rs:76-116, modal.rs:61-129).
  implication: Esc se routuje/konzumuje mimo guard modal, protože modal není skutečně keyboard owner během flow.

## Resolution

root_cause: Guard dialog pro unsaved close není zapojen do `dialog_open_base` a nemá explicitní Esc handling. Současně `process_unsaved_close_guard_dialog` v každém frame volá `editor.open_file`, což nastavuje `focus_editor_requested`, a editor si při `!dialog_open` bere focus zpět. Výsledek je, že Esc jde do editoru místo do guard modalu.
fix: N/A (goal: find_root_cause_only)
verification: Diagnostika potvrzena statickou analýzou call path a focus podmínek.
files_changed: []
