# S01: Modal volby okna s guard flow a workspace reinicializací

**Goal:** Po výběru složky / vytvoření projektu / kliknutí na nedávný projekt se zobrazí modal s volbou "Nové okno / Stávající okno / Zrušit". Obě cesty fungují kompletně včetně unsaved guard.
**Demo:** Menu → Otevřít projekt → vybrat složku → modal → "Stávající okno" → guard (pokud dirty) → starý projekt se nahradí novým.

## Must-Haves

- `pending_open_choice: Option<PathBuf>` na WorkspaceState
- `AppAction::OpenWithChoice(PathBuf)` pro wizard callback
- `show_open_choice_modal()` se 3 tlačítky (Nové okno výchozí, Stávající okno, Zrušit)
- Přepojení open_project, open_recent, wizard callback na pending stav
- `PendingCloseMode::SwitchProject(PathBuf)` pro guard flow s post-guard reinit
- Post-guard signál: po save/discard → `open_here_path = Some(path)`
- Escape/klik mimo modal = Cancel (vyčistí pending_open_choice)
- i18n klíče ve všech 5 jazycích
- `cargo check` + `./check.sh` pass

## Observability / Diagnostics

- `pending_open_choice` na `WorkspaceState` — inspektovatelný stav; pokud je `Some(path)`, modal je viditelný
- Toast chyby při selhání open_here flow se propagují přes stávající toast systém
- `OpenWithChoice` akce loguje se přes stávající `AppAction` flow v `process_actions()` — debugovatelné přes breakpoint na matchi
- Guard flow (`PendingCloseMode::SwitchProject`) se chová identicky jako existující `WorkspaceClose` — sledovatelné přes `pending_close_flow` field
- Escape/backdrop cancel čistí `pending_open_choice` → modal zmizí ihned (vizuálně pozorovatelné)
- Žádné secrets se nelogují

## Verification

- `cargo check` — žádné compile errory
- `./check.sh` — 229+ testů pass, fmt OK, clippy OK
- `grep -r 'pending_open_choice' src/` — field existuje a je používán
- `grep -r 'SwitchProject' src/` — guard mode variant existuje
- `grep -r 'OpenWithChoice' src/` — AppAction variant existuje
- `grep -c 'open-choice' locales/*/ui.ftl` — klíče ve všech 5 jazycích

## Tasks

- [x] **T01: Pending stav, modal a napojení tří entry pointů** `est:45m`
  - Why: Základ celého flow — modal se musí zobrazit po třech akcích a "Nové okno" + "Stávající okno" (bez dirty) musí fungovat
  - Files: `src/app/types.rs`, `src/app/mod.rs`, `src/app/ui/workspace/state/mod.rs`, `src/app/ui/workspace/state/types.rs`, `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/workspace/modal_dialogs.rs`, `src/app/ui/workspace/mod.rs`
  - Do:
    1. Přidat `OpenWithChoice(PathBuf)` do `AppAction` enum v `types.rs`
    2. Přidat `pending_open_choice: Option<PathBuf>` do `WorkspaceState` (nebo vhodný struct v `state/types.rs`)
    3. Vytvořit `show_open_choice_modal()` — `egui::Modal` se 3 tlačítky, vrací enum `OpenChoice { NewWindow, CurrentWindow, Cancelled, Pending }`. Výchozí (zvýrazněné) je "Nové okno". Pattern z `show_unsaved_close_guard_dialog()` (3 tlačítka přes `ui_footer_actions`).
    4. V `menubar/mod.rs` — `open_project` handler: po `try_recv()` s výsledkem nastavit `pending_open_choice = Some(path)` místo `in_new_window = true` dispatch
    5. V `menubar/mod.rs` — `open_recent` handler: nastavit `pending_open_choice = Some(path)` místo přímého `AppAction::OpenInNewWindow`
    6. V `modal_dialogs.rs` — wizard callback: nahradit `AppAction::OpenInNewWindow(path)` za `AppAction::OpenWithChoice(path)`
    7. V `app/mod.rs` — `process_actions()`: handler pro `OpenWithChoice` — nastavit `pending_open_choice` na příslušném workspace
    8. V `workspace/mod.rs` — renderovat modal pokud `pending_open_choice.is_some()`: NewWindow → push `AppAction::OpenInNewWindow` + clear pending; CurrentWindow bez dirty tabs → `open_here_path = Some(path)` + clear pending; CurrentWindow s dirty tabs → zatím jen `open_here_path` (guard v T02); Cancelled/Escape → clear pending
    9. Ošetřit `should_close()` na `egui::Modal` — Escape/klik mimo = clear pending_open_choice
  - Verify: `cargo check` pass; modal se zobrazí po open_project/open_recent/wizard; "Nové okno" otevře viewport; "Stávající okno" bez dirty přepne workspace
  - Done when: `cargo check` čistý, všechny tři entry pointy nastavují `pending_open_choice`, modal renderován a obě cesty (new window, current window bez dirty) fungují

- [x] **T02: Guard flow rozšíření, post-guard reinit a i18n** `est:30m`
  - Why: Volba "Stávající okno" s neuloženými změnami musí zobrazit unsaved guard a po save/discard provést reinicializaci
  - Files: `src/app/ui/workspace/state/mod.rs`, `src/app/ui/workspace/mod.rs`, `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl`
  - Do:
    1. Přidat `SwitchProject(PathBuf)` do `PendingCloseMode` enum v `state/mod.rs`
    2. V `workspace/mod.rs` — modal CurrentWindow s dirty tabs: místo přímého `open_here_path` spustit `pending_close_flow = Some(PendingCloseFlow::new(SwitchProject(path), dirty_tabs))`
    3. V `process_unsaved_close_guard_dialog()` — po dokončení flow (všechny tabs vyřešeny) pro `SwitchProject(path)` mode: nastavit `open_here_path = Some(path)` místo close akce. Cancel → vyčistit `pending_open_choice`.
    4. Blokovat modal pokud `pending_close_flow.is_some()` — zabránit kolizi dvou guard flow
    5. Přidat i18n klíče do `locales/*/ui.ftl` (5 jazyků):
       - `open-choice-title` — "Otevřít projekt" / "Open Project" / ...
       - `open-choice-new-window` — "Nové okno" / "New Window" / ...
       - `open-choice-current-window` — "Stávající okno" / "Current Window" / ...
       - `open-choice-cancel` — "Zrušit" / "Cancel" / ...
       - `open-choice-description` — "Kde chcete projekt otevřít?" / "Where to open the project?" / ...
    6. Napojit i18n klíče v `show_open_choice_modal()` (z T01) místo hardcoded stringů
  - Verify: `cargo check` pass; `./check.sh` pass (229+ testů); `grep -r 'SwitchProject' src/` — variant existuje; `grep -c 'open-choice' locales/*/ui.ftl` → ≥4 per jazyk; guard flow s dirty tabs → Save uloží a přepne, Cancel vrátí, Discard přepne bez uložení
  - Done when: `./check.sh` zelený, guard flow kompletní pro SwitchProject mode, i18n ve všech 5 jazycích

## Files Likely Touched

- `src/app/types.rs` — nový AppAction::OpenWithChoice
- `src/app/mod.rs` — handler pro OpenWithChoice v process_actions()
- `src/app/ui/workspace/state/mod.rs` — PendingCloseMode::SwitchProject
- `src/app/ui/workspace/state/types.rs` — pending_open_choice field
- `src/app/ui/workspace/menubar/mod.rs` — přepojení open_project a open_recent
- `src/app/ui/workspace/modal_dialogs.rs` — wizard callback přepojení, show_open_choice_modal()
- `src/app/ui/workspace/mod.rs` — modal rendering, guard flow rozšíření, post-guard reinit
- `locales/cs/ui.ftl` — i18n klíče
- `locales/en/ui.ftl` — i18n klíče
- `locales/sk/ui.ftl` — i18n klíče
- `locales/de/ui.ftl` — i18n klíče
- `locales/ru/ui.ftl` — i18n klíče
