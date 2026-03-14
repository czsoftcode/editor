---
estimated_steps: 9
estimated_files: 7
---

# T01: Pending stav, modal a napojení tří entry pointů

**Slice:** S01 — Modal volby okna s guard flow a workspace reinicializací
**Milestone:** M007

## Description

Zavést `pending_open_choice: Option<PathBuf>` jako centrální stav pro "uživatel vybral cestu, čeká se na rozhodnutí kam otevřít". Vytvořit `show_open_choice_modal()` se 3 tlačítky. Přepojit všechny tři entry pointy (open_project, open_recent, wizard) na pending stav místo přímého dispatch. Modal renderovat v `render_workspace()` — NewWindow → AppAction::OpenInNewWindow, CurrentWindow bez dirty → open_here_path.

## Steps

1. Přidat `OpenWithChoice(PathBuf)` do `AppAction` enum v `src/app/types.rs`
2. Přidat `pending_open_choice: Option<PathBuf>` do vhodného struct — pravděpodobně `WorkspaceState` v `state/mod.rs` nebo `types.rs` (ověřit kam patří dle konvence)
3. Vytvořit `show_open_choice_modal()` v `modal_dialogs.rs` (nebo nový soubor pokud lepší):
   - `egui::Modal` s title a 3 tlačítky
   - Vrací `OpenChoice` enum: `NewWindow`, `CurrentWindow`, `Cancelled`, `Pending`
   - Výchozí/zvýrazněný button je "Nové okno"
   - Pattern: `show_unsaved_close_guard_dialog()` v `confirm.rs` (3 tlačítka přes footer actions)
   - Hardcoded stringy zatím OK (i18n v T02)
4. V `menubar/mod.rs` — `folder_pick_rx` handler (cca řádek 230-268): po `try_recv()` s Ok(Some(path)) nastavit `ws.pending_open_choice = Some(path)` místo `in_new_window = true` a dispatch
5. V `menubar/mod.rs` — `open_recent` handler (cca řádek 220-228): nastavit `ws.pending_open_choice = Some(path)` místo `actions.push(AppAction::OpenInNewWindow(path))`
6. V `modal_dialogs.rs` — wizard callback: nahradit `actions.lock().unwrap().push(AppAction::OpenInNewWindow(path))` za `actions.lock().unwrap().push(AppAction::OpenWithChoice(path))`
7. V `app/mod.rs` — `process_actions()`: přidat handler pro `AppAction::OpenWithChoice(path)` — najít příslušný workspace a nastavit `pending_open_choice = Some(path)`
8. V `workspace/mod.rs` — po menu/dialog renderingu: pokud `ws.pending_open_choice.is_some()`, volat `show_open_choice_modal()`. Zpracovat výsledek:
   - `NewWindow` → push `AppAction::OpenInNewWindow(path)`, `pending_open_choice = None`
   - `CurrentWindow` → `open_here_path = Some(path)`, `pending_open_choice = None` (dirty tabs guard v T02)
   - `Cancelled` → `pending_open_choice = None`
   - `Pending` → nic (modal zůstává)
9. Ošetřit `egui::Modal::should_close()` — pokud true a stále pending, nastavit `pending_open_choice = None`

## Must-Haves

- [ ] `OpenWithChoice(PathBuf)` v AppAction enum
- [ ] `pending_open_choice: Option<PathBuf>` na WorkspaceState
- [ ] `show_open_choice_modal()` vrací OpenChoice enum
- [ ] open_project handler nastavuje pending místo přímého dispatch
- [ ] open_recent handler nastavuje pending místo přímého OpenInNewWindow
- [ ] wizard callback pushuje OpenWithChoice místo OpenInNewWindow
- [ ] Modal se renderuje v render_workspace() a reaguje na volby
- [ ] Escape/klik mimo = Cancel (clear pending)

## Observability Impact

- **Nový stav:** `pending_open_choice: Option<PathBuf>` na `WorkspaceState` — inspektovatelný přes debugger/log; `Some(path)` = modal je zobrazen, `None` = modal skrytý
- **Nová akce:** `AppAction::OpenWithChoice(PathBuf)` — prochází centrální `process_actions()`, trasovatelná jako ostatní akce
- **Vizuální signál:** Modal se 3 tlačítky je viditelný okamžitě po výběru cesty — budoucí agent ověří přes `grep -r 'pending_open_choice' src/`
- **Failure visibility:** Pokud `pending_open_choice` zůstane `Some` déle než 1 frame bez modalu = bug v renderingu (detektovatelné přes stav fieldu)
- **Cancel path:** Escape/backdrop vždy vyčistí pending stav → deterministické chování

## Verification

- `cargo check` — žádné compile errory
- `grep -r 'pending_open_choice' src/` — field existuje a je přiřazován ve 3+ místech
- `grep -r 'OpenWithChoice' src/` — variant existuje v types.rs a je zpracován v app/mod.rs

## Inputs

- `src/app/ui/workspace/menubar/mod.rs` — stávající open_project a open_recent handlery
- `src/app/ui/workspace/modal_dialogs.rs` — wizard callback
- `src/app/ui/dialogs/confirm.rs` — vzor pro 3-tlačítkový dialog (show_unsaved_close_guard_dialog)
- `src/app/ui/widgets/modal.rs` — show_modal pattern
- `src/app/types.rs` — AppAction enum
- Research: data flow, řádky 220-268 menubar, řádek 56-62 modal_dialogs

## Expected Output

- `src/app/types.rs` — rozšířen o `OpenWithChoice(PathBuf)`
- `src/app/ui/workspace/state/` — `pending_open_choice: Option<PathBuf>` field
- `src/app/ui/workspace/modal_dialogs.rs` — `show_open_choice_modal()` + `OpenChoice` enum + wizard callback přepojení
- `src/app/ui/workspace/menubar/mod.rs` — open_project a open_recent přepojeny na pending
- `src/app/ui/workspace/mod.rs` — modal rendering a result handling
- `src/app/mod.rs` — OpenWithChoice handler v process_actions()
