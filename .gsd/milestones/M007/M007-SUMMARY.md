---
id: M007
provides:
  - Modal dialog s volbou "Nové okno / Stávající okno / Zrušit" po výběru složky, vytvoření projektu ve wizardu nebo kliknutí na nedávný projekt
  - PendingCloseMode::SwitchProject(PathBuf) — rozšíření guard flow pro workspace reinicializaci místo zavření
  - AppAction::OpenWithChoice(PathBuf) — data path pro wizard callback bez přímého &mut WorkspaceState
  - pending_open_choice jako centrální stav na WorkspaceState signalizující pending modal
  - i18n open-choice-* klíče v 5 jazycích (cs, en, sk, de, ru)
key_decisions:
  - "M007 jako single-slice milestone — modal, guard rozšíření, reinicializace a i18n tvoří jeden koherentní vertikální flow"
  - "pending_open_choice: Option<PathBuf> na WorkspaceState — centrální stav pro pending modal volby"
  - "Menubar handlery nastavují pending_open_choice přímo (mají &mut WorkspaceState), wizard callback přes AppAction::OpenWithChoice"
  - "PendingCloseMode::SwitchProject(PathBuf) rozšíření guard flow — post-guard akce nastaví open_here_path místo close"
  - "Modal blokován pokud pending_close_flow.is_some() — prevence kolize dvou guard flow"
  - "Workspace reinicializace přes existující open_here_path pattern — Rust Drop automaticky zajistí cleanup"
patterns_established:
  - pending_open_choice jako centrální stav pro "čeká se na rozhodnutí kam otevřít"
  - PendingCloseMode s PathBuf payload pro guard flow s post-guard reinicializací místo zavření
  - Modal guard kolize — pending_close_flow.is_some() blokuje open choice modal rendering
observability_surfaces:
  - pending_open_choice field na WorkspaceState — Some(path) = modal zobrazen, None = skrytý
  - pending_close_flow.mode == SwitchProject(path) — signalizuje guard pro přepnutí projektu
  - process_unsaved_close_guard_dialog vrací Option<PathBuf> po Finished pro SwitchProject
  - OpenWithChoice akce prochází centrální process_actions() — trasovatelná breakpointem
requirement_outcomes:
  - id: R037
    from_status: active
    to_status: validated
    proof: "open_project handler nastavuje pending_open_choice, show_open_choice_modal() se 3 tlačítky přes StandardModal, modal renderován v render_workspace(). cargo check + ./check.sh pass."
  - id: R038
    from_status: active
    to_status: validated
    proof: "Wizard callback pushuje AppAction::OpenWithChoice(path), handler v process_actions nastavuje pending_open_choice na root_ws."
  - id: R039
    from_status: active
    to_status: validated
    proof: "open_recent handler nastavuje pending_open_choice přímo (má &mut WorkspaceState)."
  - id: R040
    from_status: active
    to_status: validated
    proof: "PendingCloseMode::SwitchProject(PathBuf) variant, PendingCloseFlow s dirty tabs queue, process_unsaved_close_guard_dialog vrací Option<PathBuf>, Cancel čistí pending_open_choice."
  - id: R041
    from_status: active
    to_status: validated
    proof: "open_here_path = Some(path) po guard/přímém výběru, existující init_workspace() pattern v app/mod.rs. Rust Drop na starém ws zajistí cleanup."
  - id: R042
    from_status: active
    to_status: validated
    proof: "5 open-choice-* klíčů (title, description, new-window, current-window, cancel) × 5 locale souborů. show_open_choice_modal() používá i18n.get()."
  - id: R043
    from_status: active
    to_status: validated
    proof: "Rust Drop na starém WorkspaceState — Terminal::drop() volá kill_process_group(), ProjectWatcher se dropne s notify::RecommendedWatcher, git_cancel.store(true)."
duration: 27m
verification_result: passed
completed_at: 2026-03-14
---

# M007: Dialog Otevření Projektu — Stávající vs Nové Okno

**Po výběru složky, vytvoření projektu ve wizardu nebo kliknutí na nedávný projekt se zobrazí 3-tlačítkový modal (Nové okno / Stávající okno / Zrušit). "Nové okno" otevře nový viewport. "Stávající okno" spustí unsaved guard pokud existují dirty tabs, po guard dokončení provede workspace reinicializaci přes existující open_here_path pattern. i18n kompletní v 5 jazycích.**

## What Happened

Milestone M007 byl realizován jako jeden slice (S01) se dvěma tasky.

**T01 (15 min)** zavedl základní infrastrukturu: `pending_open_choice: Option<PathBuf>` na `WorkspaceState` jako centrální stav signalizující zobrazení modalu. Vytvořen `show_open_choice_modal()` se 3 tlačítky přes `StandardModal` a `OpenChoice` enum (NewWindow/CurrentWindow/Cancelled/Pending). Přepojeny tři entry pointy — open_project (`folder_pick_rx.try_recv()` nastavuje `pending_open_choice`), open_recent (nastavuje přímo), wizard callback (pushuje `AppAction::OpenWithChoice(path)`). Modal rendering v `render_workspace()` zpracovává volby: NewWindow → `OpenInNewWindow`, CurrentWindow bez dirty → `open_here_path`, Cancelled → clear.

**T02 (12 min)** rozšířil guard flow: `PendingCloseMode::SwitchProject(PathBuf)` jako nová varianta pro guard s post-guard reinicializací. CurrentWindow s dirty tabs vytváří `PendingCloseFlow` se `SwitchProject` modem. `process_unsaved_close_guard_dialog()` změněna na `Option<PathBuf>` — pro SwitchProject Finished vrací cestu, call site ji slučuje do `open_here_path`. Cancel v guard čistí `pending_open_choice`. Modal guard kolize: `pending_close_flow.is_some()` blokuje open choice modal. i18n klíče přidány do všech 5 locale souborů.

Klíčové architektonické rozhodnutí: znovupoužití existujícího `open_here_path` mechanismu pro workspace reinicializaci místo nového cleanup kódu. Rust Drop na starém WorkspaceState automaticky ukončí terminály, watchers a git operace.

## Cross-Slice Verification

Single-slice milestone — žádná cross-slice integrace.

**Success Criteria:**

1. **Menu → Otevřít projekt → vybrat složku → modal se zobrazí** ✅ — `menubar/mod.rs:229` nastavuje `pending_open_choice`, modal renderován v `workspace/mod.rs:573`
2. **Menu → Nový projekt → wizard → vytvořit → stejný modal** ✅ — `modal_dialogs.rs:61` pushuje `OpenWithChoice`, handler v `app/mod.rs:360-364` nastaví `pending_open_choice`
3. **Menu → Nedávné → klik na projekt → stejný modal** ✅ — `menubar/mod.rs:245` nastavuje `pending_open_choice`
4. **Volba "Nové okno" → nový viewport** ✅ — `workspace/mod.rs:577` — `pending_open_choice.take()` → `OpenInNewWindow`
5. **Volba "Stávající okno" s neuloženými změnami → unsaved guard** ✅ — `workspace/mod.rs:600-602` — `SwitchProject(path)` s dirty tabs
6. **Volba "Stávající okno" bez neuložených změn → workspace reinit** ✅ — `workspace/mod.rs:584` — `open_here_path = Some(path)`
7. **Guard → Cancel → nic se nestane** ✅ — `workspace/mod.rs:419-422` — Cancel čistí `pending_open_choice`
8. **Guard → Save → uloží → přepne** ✅ — `workspace/mod.rs:320-322` — SwitchProject Finished → `open_here_path`
9. **Guard → Discard → přepne bez uložení** ✅ — `workspace/mod.rs:412-415` — SwitchProject cesta extrahována
10. **Terminály starého projektu se ukončí** ✅ — Rust Drop na WorkspaceState: Terminal::drop(), ProjectWatcher drop, git_cancel.store(true)
11. **`cargo check` + `./check.sh` projde čistě** ✅ — Ověřeno: 0 compile errors, all tests pass, clippy + fmt OK
12. **i18n klíče ve všech 5 jazycích** ✅ — `grep -c 'open-choice' locales/*/ui.ftl` → 5 per jazyk × 5 jazyků
13. **Escape/klik mimo modal = Cancel** ✅ — `workspace/mod.rs:610` — Cancelled varianta čistí pending stav

**Definition of Done:**

- [x] Modal dialog se zobrazí po všech třech akcích (open project, new project wizard, recent)
- [x] "Nové okno" vytvoří nový viewport
- [x] "Stávající okno" provede workspace reinicializaci
- [x] Unsaved changes guard se zobrazí při dirty tabs a správně reaguje na Save/Discard/Cancel
- [x] i18n klíče existují ve všech 5 jazycích
- [x] `cargo check` + `./check.sh` projde čistě
- [x] Escape/klik mimo modal = Cancel

## Requirement Changes

- R037: active → validated — open_project handler → pending_open_choice → modal se 3 tlačítky, cargo check + ./check.sh pass
- R038: active → validated — wizard callback → OpenWithChoice → pending_open_choice na root_ws
- R039: active → validated — open_recent handler → pending_open_choice přímo
- R040: active → validated — SwitchProject guard flow, Save/Discard/Cancel cesty implementovány
- R041: active → validated — open_here_path → init_workspace(), Rust Drop cleanup
- R042: active → validated — 5 open-choice-* klíčů × 5 locale souborů
- R043: active → validated — Rust Drop: Terminal::drop(), ProjectWatcher drop, git_cancel.store(true)

## Forward Intelligence

### What the next milestone should know
- Všechny tři projektové entry pointy (open, new, recent) nyní procházejí pending_open_choice modal — jakýkoliv nový entry point (např. drag & drop) musí nastavit pending_open_choice stejným vzorem
- Workspace reinicializace je plně delegována na existující `open_here_path` pattern v `app/mod.rs` — žádný nový cleanup kód, Rust ownership model zajišťuje korektní Drop
- Guard flow má nyní 2 mody: `WorkspaceClose` (původní) a `SwitchProject(PathBuf)` (nový) — přidání třetího modu je přímočaré

### What's fragile
- `PendingCloseMode` bez `Copy` derive kvůli `PathBuf` v `SwitchProject` — nový variant s Clone-only typem by vyžadoval clone() na copy sites (aktuálně žádné neexistují)
- Modal guard kolize (`pending_close_flow.is_some()` blokuje open choice) — pokud by se přidal třetí guard flow typ, podmínka musí být rozšířena
- `process_unsaved_close_guard_dialog()` vrací `Option<PathBuf>` — návratový typ se může dále komplikovat pokud přibudou další guard mody

### Authoritative diagnostics
- `pending_open_choice` field na WorkspaceState — Some(path) = modal zobrazen, None = skrytý
- `pending_close_flow.mode` — SwitchProject(path) signalizuje guard pro přepnutí projektu
- `process_unsaved_close_guard_dialog` vrací Some(path) po Finished pro SwitchProject

### What assumptions changed
- Plán počítal s uniformní AppAction cestou pro všechny entry pointy — menubar handlery mají přímý přístup k `&mut WorkspaceState`, takže AppAction detour je zbytečný (OpenWithChoice použit jen pro wizard callback)
- `process_unsaved_close_guard_dialog()` změna z `()` na `Option<PathBuf>` nebyla v plánu, ale je čistější než extra field na WorkspaceState

## Files Created/Modified

- `src/app/types.rs` — přidán `OpenWithChoice(PathBuf)` do AppAction enum
- `src/app/mod.rs` — OpenWithChoice handler v process_actions, pending_open_choice v testových konstruktorech
- `src/app/ui/workspace/state/mod.rs` — `pending_open_choice` field, `SwitchProject(PathBuf)` v PendingCloseMode, odstraněn Copy derive
- `src/app/ui/workspace/state/init.rs` — inicializace `pending_open_choice: None`
- `src/app/ui/workspace/modal_dialogs.rs` — `OpenChoice` enum, `show_open_choice_modal()` s i18n, wizard callback přepojen na OpenWithChoice
- `src/app/ui/workspace/menubar/mod.rs` — open_project a open_recent přepojeny na pending
- `src/app/ui/workspace/mod.rs` — modal rendering, guard flow CurrentWindow+SwitchProject, process_unsaved_close_guard_dialog → Option<PathBuf>, modal guard podmínka
- `locales/cs/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/en/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/sk/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/de/ui.ftl` — 5 nových open-choice-* klíčů
- `locales/ru/ui.ftl` — 5 nových open-choice-* klíčů
