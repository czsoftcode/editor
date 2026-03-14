---
id: T02
parent: S01
milestone: M007
provides:
  - SwitchProject(PathBuf) varianta v PendingCloseMode enum
  - Guard flow pro dirty tabs při přepnutí projektu ve stávajícím okně
  - Cancel v guard flow čistí pending_open_choice
  - Modal guard — open choice modal se nevykresluje při aktivním pending_close_flow
  - i18n klíče open-choice-* v 5 jazycích (cs, en, sk, de, ru)
  - show_open_choice_modal() používá i18n.get() místo hardcoded stringů
key_files:
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/modal_dialogs.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - PendingCloseMode ztratil Copy derive kvůli PathBuf v SwitchProject — existující kód nekopíroval mode přímo, takže žádný breaking change
  - process_unsaved_close_guard_dialog() vrací Option<PathBuf> — pro SwitchProject Finished vrací cestu, jinak None. Call site v render_workspace slučuje s open_here_path
  - SwitchProject path se extrahuje z flow.mode těsně před vynulováním pending_close_flow — obě Finished branches (hlavní a tab-no-longer-exists) to ošetřují
  - Cancel v guard flow čistí pending_open_choice na None — brání re-popup modalu
patterns_established:
  - Guard flow pro project switch — identický pattern jako WorkspaceClose, ale po Finished nastaví open_here_path místo close akce
  - Modal guard kolize — pending_close_flow.is_some() blokuje open choice modal rendering
observability_surfaces:
  - pending_close_flow.mode == SwitchProject(path) — inspektovatelné přes debugger, signalizuje guard pro přepnutí projektu
  - process_unsaved_close_guard_dialog vrací Some(path) po Finished pro SwitchProject — trasovatelné přes breakpoint na return
  - pending_open_choice == None po Cancel — ověřitelné přes debugger
  - i18n klíče ověřitelné přes grep -c 'open-choice' locales/*/ui.ftl
duration: 12m
verification_result: passed
completed_at: 2026-03-14
blocker_discovered: false
---

# T02: Guard flow rozšíření, post-guard reinit a i18n

**Přidání SwitchProject guard flow pro dirty tabs při přepnutí projektu, cancel cleanup, modal guard prevence kolize, a i18n napojení open choice modalu v 5 jazycích.**

## What Happened

1. Přidán `SwitchProject(PathBuf)` do `PendingCloseMode` enum. Odstraněn `Copy` derive (PathBuf není Copy) — bez breaking changes, stávající kód mode nekopíroval.

2. CurrentWindow branch v open choice modalu: pokud existují dirty tabs, vytvoří `PendingCloseFlow` s `SwitchProject(path)` modem a frontu dirty tabs (WorkspaceClose queue pattern). Pokud ne → rovnou `open_here_path = Some(path)`.

3. `process_unsaved_close_guard_dialog()` změněna na `-> Option<PathBuf>`. Po Finished pro SwitchProject extrahuje cestu z `flow.mode` a vrátí ji. Call site v `render_workspace` ji sloučí do `open_here_path`. Obě Finished branches (hlavní outcome a tab-no-longer-exists fallback) ošetřují SwitchProject.

4. Cancel v guard flow vyčistí `pending_open_choice = None` — brání re-popup modalu po zrušení.

5. Modal guard: `pending_close_flow.is_some()` podmínka přidána k open choice modal renderingu — prevence kolize dvou modálů.

6. i18n klíče `open-choice-title`, `open-choice-description`, `open-choice-new-window`, `open-choice-current-window`, `open-choice-cancel` přidány do všech 5 locale souborů.

7. `show_open_choice_modal()` přijímá `&I18n` parametr a používá `i18n.get("open-choice-*")` místo hardcoded českých stringů.

## Verification

- `cargo check` — žádné compile errory ✅
- `./check.sh` — 192 testů pass, fmt OK, clippy OK ✅
- `grep -r 'SwitchProject' src/` — variant existuje, zpracován v process_unsaved_close_guard_dialog ve 2 Finished branches ✅
- `grep -c 'open-choice' locales/cs/ui.ftl` — 5 ✅
- `grep -c 'open-choice' locales/en/ui.ftl` — 5 ✅
- `grep -c 'open-choice' locales/sk/ui.ftl` — 5 ✅
- `grep -c 'open-choice' locales/de/ui.ftl` — 5 ✅
- `grep -c 'open-choice' locales/ru/ui.ftl` — 5 ✅

### Slice-level verifikace (mezistav):
- ✅ `cargo check` pass
- ✅ `./check.sh` 192 testů pass
- ✅ `pending_open_choice` existuje a je používán
- ✅ `SwitchProject` existuje a je zpracován
- ✅ `OpenWithChoice` existuje
- ✅ i18n klíče `open-choice` ve všech 5 jazycích

## Diagnostics

- `ws.pending_close_flow.mode` — pokud je `SwitchProject(path)`, guard flow běží pro přepnutí projektu
- `process_unsaved_close_guard_dialog` vrací `Some(path)` po Finished pro SwitchProject → trasovatelné přes breakpoint na return
- `pending_open_choice = None` po Cancel — modal zmizí ihned
- `pending_close_flow.is_some() && pending_open_choice.is_some()` = guard má prioritu, open choice modal se nevykresluje

## Deviations

- `PendingCloseMode` ztratil `Copy` derive — nutné kvůli PathBuf, ale neovlivnilo stávající kód
- `process_unsaved_close_guard_dialog` změněna z `()` na `Option<PathBuf>` — čistší než alternativy (extra field na WorkspaceState nebo mutable reference)

## Known Issues

- Žádné

## Files Created/Modified

- `src/app/ui/workspace/state/mod.rs` — přidán `SwitchProject(PathBuf)` do PendingCloseMode, odstraněn Copy derive
- `src/app/ui/workspace/mod.rs` — CurrentWindow guard flow, process_unsaved_close_guard_dialog vrací Option<PathBuf>, SwitchProject handling v obou Finished branches, Cancel čistí pending_open_choice, modal guard podmínka, i18n parametr pro show_open_choice_modal
- `src/app/ui/workspace/modal_dialogs.rs` — show_open_choice_modal přijímá &I18n, nahrazeny hardcoded stringy za i18n.get()
- `locales/cs/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/en/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/sk/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/de/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/ru/ui.ftl` — 5 nových open-choice-* klíčů
- `.gsd/milestones/M007/slices/S01/tasks/T02-PLAN.md` — přidána Observability Impact sekce
