---
id: T01
parent: S01
milestone: M007
provides:
  - pending_open_choice field na WorkspaceState
  - OpenWithChoice AppAction variant
  - show_open_choice_modal() se 3 tlačítky
  - Přepojení open_project, open_recent, wizard na pending stav
  - Modal rendering v render_workspace s dispatch logiku
key_files:
  - src/app/types.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/modal_dialogs.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/mod.rs
  - src/app/mod.rs
key_decisions:
  - Menubar handlery (open_project, open_recent) nastavují ws.pending_open_choice přímo, bez AppAction — mají přímý přístup k WorkspaceState
  - Wizard callback pushuje AppAction::OpenWithChoice protože nemá přímý přístup k ws
  - OpenWithChoice handler v process_actions nastavuje pending na root_ws (sekundární okna nejsou relevantní)
  - Modal použuje StandardModal s ui_footer (ne ui_footer_actions) — hardcoded stringy se nahradí i18n v T02
patterns_established:
  - pending_open_choice jako centrální stav pro "čeká se na rozhodnutí kam otevřít"
  - Modal vrací enum (NewWindow/CurrentWindow/Cancelled/Pending) zpracovávaný v render_workspace
observability_surfaces:
  - pending_open_choice field na WorkspaceState — Some(path) = modal zobrazen, None = skrytý
  - OpenWithChoice akce prochází centrální process_actions() — trasovatelná breakpointem
duration: 15m
verification_result: passed
completed_at: 2026-03-14
blocker_discovered: false
---

# T01: Pending stav, modal a napojení tří entry pointů

**Zavedení `pending_open_choice` jako centrálního stavu pro výběr kam otevřít projekt, vytvoření 3-tlačítkového modalu a přepojení všech tří entry pointů na pending flow.**

## What Happened

1. Přidán `OpenWithChoice(PathBuf)` do `AppAction` enum — nová akce pro výběr cesty, kde ještě není rozhodnuto kam otevřít.

2. Přidán `pending_open_choice: Option<PathBuf>` na `WorkspaceState` — centrální stav signalizující že modal má být zobrazený. Inicializován na `None` v `init_workspace()` a obou testovacích konstruktorech.

3. Vytvořen `show_open_choice_modal()` v `modal_dialogs.rs` + `OpenChoice` enum (`NewWindow`, `CurrentWindow`, `Cancelled`, `Pending`). Modal používá `StandardModal` s backdrop, Escape consumuje a vrací `Cancelled`. Hardcoded české stringy — i18n v T02.

4. Open_project handler v menubar přepojen: `folder_pick_rx` po `try_recv()` nastavuje `ws.pending_open_choice = Some(path)` místo přímého `OpenInNewWindow`. Parametr `in_new_window` je ignorován (podtržítko).

5. Open_recent handler přepojen: pushuje `AppAction::AddRecent` ale místo `OpenInNewWindow` nastavuje `ws.pending_open_choice` přímo.

6. Wizard callback přepojen: pushuje `AppAction::OpenWithChoice(path)` místo `OpenInNewWindow`.

7. `process_actions()` v `app/mod.rs`: handler pro `OpenWithChoice` nastaví `pending_open_choice` na `root_ws`.

8. Modal rendering v `render_workspace()`: po `render_replace_preview_dialog`, pokud `pending_open_choice.is_some()`, volá `show_open_choice_modal()`. Zpracování: NewWindow → push `OpenInNewWindow` + clear, CurrentWindow → `open_here_path = Some(path)` + clear, Cancelled → clear.

9. `pending_open_choice` přidáno do `dialog_open_base` → editor je zamčený během modalu.

## Verification

- `cargo check` — žádné compile errory ani warningy ✅
- `./check.sh` — 192 testů pass, fmt OK, clippy OK ✅
- `grep -r 'pending_open_choice' src/` — 13 výskytů, field existuje, je přiřazován v 4+ místech ✅
- `grep -r 'OpenWithChoice' src/` — variant v types.rs, wizard callback, process_actions ✅

### Slice-level verifikace (mezistav):
- ✅ `cargo check` pass
- ✅ `./check.sh` 192 testů pass
- ✅ `pending_open_choice` existuje (13 výskytů)
- ❌ `SwitchProject` — T02
- ✅ `OpenWithChoice` existuje
- ❌ i18n klíče `open-choice` — T02

## Diagnostics

- `ws.pending_open_choice` je inspektovatelný přes debugger — `Some(path)` = modal zobrazen
- `AppAction::OpenWithChoice` prochází centrální `process_actions()` — breakpoint na matchi

## Deviations

- Menubar handlery (open_project, open_recent) nastavují `ws.pending_open_choice` přímo místo přes `AppAction::OpenWithChoice` — plán počítal s tím, že vše půjde přes AppAction, ale přímý přístup k `ws` je čistší a nevyžaduje hledání workspace v `process_actions()`.
- `folder_pick_rx` handler už nerozlišuje `in_new_window` flag — obojí jde přes modal. Parametr je podtržítkový.

## Known Issues

- Žádné

## Files Created/Modified

- `src/app/types.rs` — přidán `OpenWithChoice(PathBuf)` do AppAction enum
- `src/app/ui/workspace/state/mod.rs` — přidán `pending_open_choice: Option<PathBuf>` na WorkspaceState
- `src/app/ui/workspace/state/init.rs` — inicializace `pending_open_choice: None`
- `src/app/ui/workspace/modal_dialogs.rs` — `OpenChoice` enum, `show_open_choice_modal()`, wizard callback přepojen na OpenWithChoice
- `src/app/ui/workspace/menubar/mod.rs` — open_project a open_recent přepojeny na pending
- `src/app/ui/workspace/mod.rs` — import OpenChoice/show_open_choice_modal, modal rendering v render_workspace, pending v dialog_open_base
- `src/app/mod.rs` — OpenWithChoice handler v process_actions, pending_open_choice v testových konstruktorech
- `.gsd/milestones/M007/slices/S01/S01-PLAN.md` — přidána Observability / Diagnostics sekce
- `.gsd/milestones/M007/slices/S01/tasks/T01-PLAN.md` — přidána Observability Impact sekce
