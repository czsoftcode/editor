---
estimated_steps: 6
estimated_files: 7
---

# T02: Guard flow rozšíření, post-guard reinit a i18n

**Slice:** S01 — Modal volby okna s guard flow a workspace reinicializací
**Milestone:** M007

## Description

Rozšířit `PendingCloseMode` o `SwitchProject(PathBuf)` variantu. Upravit `process_unsaved_close_guard_dialog()` tak, aby po dokončení guard flow (save/discard pro všechny dirty tabs) nastavil `open_here_path` místo close akce. Přidat i18n klíče a napojit je v modalu z T01. Zajistit, že Cancel v guard vyčistí `pending_open_choice`.

## Steps

1. Přidat `SwitchProject(PathBuf)` do `PendingCloseMode` enum v `src/app/ui/workspace/state/mod.rs`. Exhaustive match pattern zajistí, že compiler upozorní na všechna místa kde se mode zpracovává.
2. V `workspace/mod.rs` — modal `CurrentWindow` branch: zkontrolovat jestli existují dirty (modified) tabs. Pokud ano → vytvořit `PendingCloseFlow::new(SwitchProject(path), dirty_tabs)` a nastavit `ws.pending_close_flow = Some(flow)`, clear `pending_open_choice`. Pokud ne → ponechat stávající `open_here_path = Some(path)`.
3. V `process_unsaved_close_guard_dialog()` — po dokončení flow (všechny tabs vyřešeny):
   - Pro `SwitchProject(path)` mode: nastavit `open_here_path = Some(path)` (ne close akci)
   - Pro cancel: vyčistit `pending_open_choice = None` (zabránit modal re-popup)
   - Stávající mody (TabClose, WindowClose, WorkspaceClose) se nemění
4. V modal rendering v `workspace/mod.rs` — přidat guard: pokud `ws.pending_close_flow.is_some()`, nepokouší se renderovat open choice modal (prevence kolize dvou guard flow)
5. Přidat i18n klíče do všech 5 locale souborů (`locales/*/ui.ftl`):
   - `open-choice-title = Otevřít projekt` / `Open Project` / `Otvoriť projekt` / `Projekt öffnen` / `Открыть проект`
   - `open-choice-new-window = Nové okno` / `New Window` / `Nové okno` / `Neues Fenster` / `Новое окно`
   - `open-choice-current-window = Stávající okno` / `Current Window` / `Existujúce okno` / `Aktuelles Fenster` / `Текущее окно`
   - `open-choice-cancel = Zrušit` / `Cancel` / `Zrušiť` / `Abbrechen` / `Отмена`
   - `open-choice-description = Kde chcete projekt otevřít?` / `Where do you want to open the project?` / `Kde chcete projekt otvoriť?` / `Wo möchten Sie das Projekt öffnen?` / `Где вы хотите открыть проект?`
6. Napojit i18n klíče v `show_open_choice_modal()` z T01 — nahradit hardcoded stringy za `fl!()` volání

## Must-Haves

- [ ] `SwitchProject(PathBuf)` v PendingCloseMode enum
- [ ] Guard flow po save/discard → `open_here_path` pro SwitchProject mode
- [ ] Guard cancel → vyčistí pending_open_choice
- [ ] Modal blokován pokud pending_close_flow.is_some()
- [ ] i18n klíče ve všech 5 jazycích (≥5 klíčů per jazyk)
- [ ] Modal používá i18n klíče (ne hardcoded stringy)

## Verification

- `cargo check` — čistý
- `./check.sh` — 229+ testů pass, fmt OK, clippy OK
- `grep -r 'SwitchProject' src/` — variant existuje a je zpracován v process_unsaved_close_guard_dialog
- `grep -c 'open-choice' locales/cs/ui.ftl` — ≥5 klíčů
- `grep -c 'open-choice' locales/en/ui.ftl` — ≥5 klíčů
- `grep -c 'open-choice' locales/sk/ui.ftl` — ≥5 klíčů
- `grep -c 'open-choice' locales/de/ui.ftl` — ≥5 klíčů
- `grep -c 'open-choice' locales/ru/ui.ftl` — ≥5 klíčů

## Inputs

- `src/app/ui/workspace/state/mod.rs` — PendingCloseMode enum, PendingCloseFlow struct
- `src/app/ui/workspace/mod.rs` — process_unsaved_close_guard_dialog(), modal rendering z T01
- `src/app/ui/workspace/modal_dialogs.rs` — show_open_choice_modal() z T01
- `locales/en/ui.ftl:354-358` — existující unsaved guard i18n klíče jako vzor

## Expected Output

- `src/app/ui/workspace/state/mod.rs` — rozšířen o SwitchProject(PathBuf) variant
- `src/app/ui/workspace/mod.rs` — guard flow zpracování pro SwitchProject, modal guard
- `src/app/ui/workspace/modal_dialogs.rs` — i18n napojení v modalu
- `locales/cs/ui.ftl` — 5+ nových open-choice-* klíčů
- `locales/en/ui.ftl` — 5+ nových open-choice-* klíčů
- `locales/sk/ui.ftl` — 5+ nových open-choice-* klíčů
- `locales/de/ui.ftl` — 5+ nových open-choice-* klíčů
- `locales/ru/ui.ftl` — 5+ nových open-choice-* klíčů

## Observability Impact

- `ws.pending_close_flow.mode` — pokud je `SwitchProject(path)`, guard flow běží pro přepnutí projektu; inspektovatelné přes debugger na `process_unsaved_close_guard_dialog`
- Po Finished pro SwitchProject: `process_unsaved_close_guard_dialog` vrací `Some(path)` → `open_here_path` se nastaví → workspace reinicializace proběhne standardní cestou
- Po Cancel v guard flow: `pending_open_choice` se vyčistí na `None` → modal se nezobrazí znovu; `last_unsaved_close_cancelled = true` signalizuje cancel
- Guard + modal kolize: pokud `pending_close_flow.is_some()`, open choice modal se nevykresluje — diagnostikovatelné přes `pending_open_choice.is_some() && pending_close_flow.is_some()` (oba nastavené = guard má prioritu)
- i18n klíče `open-choice-*` — ověřitelné přes `grep -c 'open-choice' locales/*/ui.ftl`
