---
id: S01
parent: M007
milestone: M007
provides:
  - pending_open_choice: Option<PathBuf> na WorkspaceState jako centrální stav pro pending modal
  - AppAction::OpenWithChoice(PathBuf) pro wizard callback data path
  - show_open_choice_modal() se 3 tlačítky (Nové okno / Stávající okno / Zrušit) s i18n
  - Přepojení open_project, open_recent, wizard callback na pending stav
  - PendingCloseMode::SwitchProject(PathBuf) pro guard flow s post-guard reinicializací
  - Modal rendering v render_workspace s dispatch logikou
  - i18n klíče open-choice-* v 5 jazycích (cs, en, sk, de, ru)
requires: []
affects: []
key_files:
  - src/app/types.rs
  - src/app/mod.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/modal_dialogs.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Menubar handlery (open_project, open_recent) nastavují ws.pending_open_choice přímo — mají &mut WorkspaceState
  - Wizard callback pushuje AppAction::OpenWithChoice protože nemá přímý přístup k ws
  - OpenWithChoice handler v process_actions nastavuje pending na root_ws
  - PendingCloseMode ztratil Copy derive kvůli PathBuf v SwitchProject — bez breaking changes
  - process_unsaved_close_guard_dialog() vrací Option<PathBuf> pro SwitchProject post-guard cestu
  - Modal blokován pokud pending_close_flow.is_some() — prevence kolize dvou guard flow
  - Cancel v guard flow čistí pending_open_choice — brání re-popup modalu
patterns_established:
  - pending_open_choice jako centrální stav pro "čeká se na rozhodnutí kam otevřít"
  - PendingCloseMode::SwitchProject pro guard flow s post-guard reinicializací místo zavření
  - Modal guard kolize — pending_close_flow.is_some() blokuje open choice modal rendering
observability_surfaces:
  - pending_open_choice field na WorkspaceState — Some(path) = modal zobrazen, None = skrytý
  - pending_close_flow.mode == SwitchProject(path) — signalizuje guard pro přepnutí projektu
  - process_unsaved_close_guard_dialog vrací Some(path) po Finished pro SwitchProject
  - OpenWithChoice akce prochází centrální process_actions() — trasovatelná breakpointem
drill_down_paths:
  - .gsd/milestones/M007/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M007/slices/S01/tasks/T02-SUMMARY.md
duration: 27m
verification_result: passed
completed_at: 2026-03-14
---

# S01: Modal volby okna s guard flow a workspace reinicializací

**Po výběru složky, vytvoření projektu ve wizardu nebo kliknutí na nedávný projekt se zobrazí 3-tlačítkový modal (Nové okno / Stávající okno / Zrušit). "Nové okno" otevře nový viewport. "Stávající okno" spustí unsaved guard pokud existují dirty tabs, po guard dokončení provede workspace reinicializaci přes existující open_here_path pattern. i18n kompletní v 5 jazycích.**

## What Happened

**T01 (15 min)** zavedl základní infrastrukturu: `pending_open_choice: Option<PathBuf>` na `WorkspaceState` jako centrální stav signalizující zobrazení modalu. Vytvořen `show_open_choice_modal()` se 3 tlačítky přes `StandardModal` a `OpenChoice` enum (NewWindow/CurrentWindow/Cancelled/Pending). Přepojeny tři entry pointy:
- **open_project** — `folder_pick_rx.try_recv()` nastavuje `pending_open_choice` místo přímého `OpenInNewWindow`
- **open_recent** — nastavuje `pending_open_choice` přímo místo `AppAction::OpenInNewWindow`
- **wizard callback** — pushuje `AppAction::OpenWithChoice(path)` místo `OpenInNewWindow`

Modal rendering v `render_workspace()` zpracovává volby: NewWindow → `AppAction::OpenInNewWindow` + clear, CurrentWindow bez dirty → `open_here_path = Some(path)` + clear, Cancelled → clear. `pending_open_choice` přidáno do `dialog_open_base` — editor zamčený během modalu.

**T02 (12 min)** rozšířil guard flow: `PendingCloseMode::SwitchProject(PathBuf)` jako nová varianta pro guard flow s post-guard reinicializací. CurrentWindow s dirty tabs vytváří `PendingCloseFlow` se `SwitchProject` modem. `process_unsaved_close_guard_dialog()` změněna z `()` na `Option<PathBuf>` — pro SwitchProject Finished vrací cestu, call site ji slučuje do `open_here_path`. Cancel v guard čistí `pending_open_choice`. Modal guard: `pending_close_flow.is_some()` blokuje open choice modal. i18n klíče `open-choice-*` přidány do všech 5 locale souborů, `show_open_choice_modal()` napojeno na `i18n.get()`.

## Verification

- `cargo check` — žádné compile errory ✅
- `./check.sh` — 192 testů pass, fmt OK, clippy OK ✅
- `grep -r 'pending_open_choice' src/` — 16 výskytů ✅
- `grep -r 'SwitchProject' src/` — 8 výskytů ✅
- `grep -r 'OpenWithChoice' src/` — 3 výskytů ✅
- `grep -c 'open-choice' locales/*/ui.ftl` — 5 per jazyk × 5 jazyků ✅

## Requirements Validated

- R037 — Modal dialog s volbou "Nové okno / Stávající okno / Zrušit" po výběru složky v "Otevřít projekt". Proof: open_project handler nastavuje pending_open_choice, modal renderován v render_workspace.
- R038 — Stejný modal po vytvoření projektu v wizardu. Proof: wizard callback pushuje OpenWithChoice, handler nastaví pending na root_ws.
- R039 — Stejný modal po kliknutí na nedávný projekt. Proof: open_recent handler nastavuje pending_open_choice.
- R040 — Unsaved changes guard při volbě "stávající okno". Proof: SwitchProject mode v PendingCloseFlow, Save/Discard/Cancel cesty implementovány, Cancel čistí pending.
- R041 — Workspace reinicializace: cleanup + nový init_workspace(). Proof: open_here_path = Some(path) po guard/přímém výběru, existující init_workspace pattern v app/mod.rs.
- R042 — i18n pro nový dialog ve všech 5 jazycích. Proof: 5 open-choice-* klíčů × 5 locale souborů.
- R043 — Terminály, watchers a background procesy se korektně ukončí. Proof: Rust Drop na starém WorkspaceState při init_workspace reinicializaci — Terminal::drop() + ProjectWatcher drop + git_cancel.store(true).

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- Menubar handlery nastavují `pending_open_choice` přímo místo přes `AppAction::OpenWithChoice` — plán počítal s uniformní cestou přes AppAction, ale přímý přístup k `&mut WorkspaceState` je čistší.
- `PendingCloseMode` ztratil `Copy` derive kvůli `PathBuf` v `SwitchProject` — bez dopadu na stávající kód.
- `process_unsaved_close_guard_dialog()` změněna z `()` na `Option<PathBuf>` — nebylo v plánu, ale čistší než extra field na WorkspaceState.

## Known Limitations

- Žádné

## Follow-ups

- none

## Files Created/Modified

- `src/app/types.rs` — přidán `OpenWithChoice(PathBuf)` do AppAction enum
- `src/app/mod.rs` — OpenWithChoice handler v process_actions, pending_open_choice v testových konstruktorech
- `src/app/ui/workspace/state/mod.rs` — `pending_open_choice` field, `SwitchProject(PathBuf)` v PendingCloseMode, odstraněn Copy derive
- `src/app/ui/workspace/state/init.rs` — inicializace `pending_open_choice: None`
- `src/app/ui/workspace/modal_dialogs.rs` — `OpenChoice` enum, `show_open_choice_modal()` s i18n, wizard callback přepojen na OpenWithChoice
- `src/app/ui/workspace/menubar/mod.rs` — open_project a open_recent přepojeny na pending
- `src/app/ui/workspace/mod.rs` — modal rendering, guard flow CurrentWindow+SwitchProject, process_unsaved_close_guard_dialog → Option<PathBuf>, modal guard podmínka, i18n parametr
- `locales/cs/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/en/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/sk/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/de/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/ru/ui.ftl` — 5 nových open-choice-* klíčů

## Forward Intelligence

### What the next slice should know
- Toto je single-slice milestone — žádná downstream slice.

### What's fragile
- `PendingCloseMode` bez Copy derive — nový variant s Clone-only typem by vyžadoval clone() na copy sites (aktuálně žádné neexistují)
- Modal guard kolize (pending_close_flow blokuje open choice) — pokud by se přidal třetí guard flow typ, podmínka musí být rozšířena

### Authoritative diagnostics
- `pending_open_choice` field na WorkspaceState — Some(path) = modal zobrazen, None = skrytý
- `pending_close_flow.mode` — SwitchProject(path) signalizuje guard pro přepnutí projektu
- `process_unsaved_close_guard_dialog` vrací Some(path) po Finished pro SwitchProject

### What assumptions changed
- Plán počítal s uniformní AppAction cestou pro všechny entry pointy — menubar handlery mají přímý přístup k ws, takže AppAction detour je zbytečný
