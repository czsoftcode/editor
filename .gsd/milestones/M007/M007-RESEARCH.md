# M007: Dialog Otevření Projektu — Stávající vs Nové Okno — Research

**Date:** 2026-03-14

## Summary

Reinicializace workspace je podstatně jednodušší než vypadalo z kontextu. Existující `open_here_path` pattern v `render_workspace()` → caller (`app/mod.rs`) **už dnes provádí kompletní `ws = init_workspace(new_path, ...)`**, přičemž starý `WorkspaceState` se dropne a Rust ownership automaticky zajistí cleanup:
- **Terminály** — `Terminal::drop()` volá `kill_process_group()` (řádek 498 `terminal/instance/mod.rs`)
- **Watchers** — `notify::RecommendedWatcher` se dropne s `ProjectWatcher` structem
- **Git cancel** — `WorkspaceState::drop()` nastaví `git_cancel.store(true)` (řádek 149 `state/mod.rs`)

Toto funguje identicky pro root workspace (řádek 863-881 `app/mod.rs`) i secondary viewport (řádek 431-451). Žádný nový cleanup kód není potřeba — stačí vrátit cestu přes `open_here_path` a existující machinery ji zpracuje.

Hlavní architektonická práce je:
1. **Nový modal s volbou** (3 tlačítka: Nové okno / Stávající okno / Zrušit) — `show_modal` má jen 2 tlačítka, potřeba buď nový helper nebo přímé `egui::Modal`
2. **Pending stav** pro cestu mezi výběrem složky a rozhodnutím v modalu — `Option<PathBuf>` na `WorkspaceState`
3. **Napojení unsaved guard** na "stávající okno" volbu — existující `PendingCloseFlow` s `WorkspaceClose` mode je navržen pro zavření, ne pro "přepni a pokračuj reinitem". Potřeba buď nový `PendingCloseMode` variant, nebo post-guard signál.

## Recommendation

**Přístup: "Pending path + synchronní modal + rozšíření guard flow"**

1. Přidat `pending_open_choice: Option<PathBuf>` na `WorkspaceState` — nastavit po výběru složky / wizard complete / recent click (místo přímého `AppAction::OpenInNewWindow` nebo `in_new_window = true`)
2. Vytvořit nový `show_open_choice_modal()` — jednoduchý `egui::Modal` se 3 tlačítky, vrací enum `OpenChoiceResult { NewWindow, CurrentWindow, Cancelled, Pending }`
3. Renderovat modal v `render_workspace()` po menu/dialog renderingu — pokud `pending_open_choice.is_some()`
4. Při `NewWindow` → push `AppAction::OpenInNewWindow` (jako dnes)
5. Při `CurrentWindow` → spustit unsaved guard (pokud dirty tabs), po guard dokončení → `open_here_path = Some(path)`
6. Guard flow: přidat nový `PendingCloseMode::SwitchProject(PathBuf)` — po dokončení se cesta vrátí jako `open_here_path` místo zavření

**Proč tento přístup:**
- Minimální invazivita — znovupoužívá `open_here_path` mechanismus, který už funguje
- Unsaved guard je otestovaný (9+ testů) — rozšíření módu je bezpečnější než nový flow
- Žádné nové runtime závislosti

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Workspace cleanup (terminály, watchers, git) | `WorkspaceState::drop()` + Rust ownership | Starý ws se dropne automaticky při `ws = init_workspace(...)` — žádný manuální cleanup |
| Workspace reinicializace | `open_here_path` → `init_workspace()` v `app/mod.rs:863-881` (root) a `431-451` (secondary) | Existující pattern, oba viewport typy pokryty |
| Unsaved changes guard | `PendingCloseFlow` + `process_unsaved_close_guard_dialog()` + `show_unsaved_close_guard_dialog()` | 9+ unit testů, Save/Discard/Cancel, inline error handling |
| Modal rendering | `egui::Modal` (nativní) + `show_modal()` helper | `show_modal` má 2 tlačítka, ale `egui::Modal` přímo podporuje custom layout — nový helper pro 3 tlačítka |
| i18n | FTL klíče v `locales/*/ui.ftl` | Existující pipeline, stačí přidat klíče |
| File dialog async | `spawn_task()` + `folder_pick_rx` + `try_recv()` | Existující pattern v `menubar/mod.rs:249-268`, nesmí se měnit |

## Existing Code and Patterns

- `src/app/ui/workspace/menubar/mod.rs:220-228` — `open_recent` handler: přímo pushuje `AppAction::OpenInNewWindow`. **Zásahový bod:** místo toho nastavit `pending_open_choice = Some(path)`.
- `src/app/ui/workspace/menubar/mod.rs:230-268` — `folder_pick_rx` processing + `open_project` spawn: `in_new_window = true` hardcoded (řádek 265). **Zásahový bod:** po `try_recv` s výsledkem nastavit `pending_open_choice` místo okamžitého dispatch.
- `src/app/ui/workspace/modal_dialogs.rs:56-62` — Wizard callback: pushuje `AppAction::OpenInNewWindow(path)`. **Zásahový bod:** nastavit `pending_open_choice` místo přímého action.
- `src/app/mod.rs:863-881` — Root workspace reinit přes `open_here_path`: `ws = init_workspace(new_path, ...)` + title update + session save. Tento kód se **nemění** — jen ho budeme volat i pro "stávající okno" volbu.
- `src/app/mod.rs:431-451` — Secondary viewport reinit: identický pattern. Také se **nemění**.
- `src/app/ui/widgets/modal.rs:278-322` — `show_modal<T>()`: 2 tlačítka (OK + Cancel). Potřeba nová varianta pro 3 tlačítka.
- `src/app/ui/dialogs/confirm.rs:78-131` — `show_unsaved_close_guard_dialog()`: 3 tlačítka přes `StandardModal` + `ui_footer_actions`. Pattern pro vlastní multi-button modal.
- `src/app/ui/workspace/state/mod.rs:35-64` — `PendingCloseMode` enum + `PendingCloseFlow` struct: rozšiřitelný o novou variantu.
- `src/app/ui/workspace/mod.rs:281-410` — `process_unsaved_close_guard_dialog()`: zpracování guard flow krok po kroku. Po dokončení flow se výsledek projeví přes `ws.last_unsaved_close_cancelled` — potřeba rozšíření pro "po guard pokračuj reinitem".
- `locales/en/ui.ftl:354-358` — Existující unsaved guard i18n klíče: `unsaved-close-guard-*`. Pattern pro nové klíče.
- `src/app/types.rs:135-144` — `AppAction` enum: `OpenInNewWindow(PathBuf)`. Není potřeba nová varianta — stačí existující `OpenInNewWindow` pro "nové okno" a `open_here_path` pro "stávající".

## Constraints

- **UI vlákno nesmí blokovat** — File dialog je asynchronní přes `spawn_task()`, modal volby je synchronní (egui frame-based). To je v pořádku — modal se renderuje po frame, ne blokujícím voláním.
- **Stávající `open_folder` (File → Open Folder) se nemění** — zůstává bez dialogu, vždy `in_new_window = false`. Výslovně mimo scope.
- **`cargo check` + `./check.sh` musí projít po každé slice** — 229+ testů zelených.
- **Žádné nové runtime závislosti** — `egui::Modal` je součást eframe.
- **Multi-viewport architektura (root_ws + secondary)** — Dialog se renderuje na workspace úrovni (ne App úrovni), takže funguje v obou viewport typech automaticky.
- **`WorkspaceState` je ne-Send** (obsahuje `LocalHistory`, `mpsc::Receiver` aj.) — reinicializace musí proběhnout na main threadu. Existující `open_here_path` → `init_workspace()` pattern to respektuje.

## Common Pitfalls

- **Guard flow nedokončí reinit** — Stávající `process_unsaved_close_guard_dialog` po dokončení nastaví `ws.last_unsaved_close_cancelled` a smaže `pending_close_flow`, ale nemá mechanismus pro "po guard pokračuj akcí X". Rozšíření musí přidat "post-guard action" field (buď na `PendingCloseFlow` nebo vedle). Pokud se zapomene, guard uloží soubory ale nepřepne projekt.
- **Wizard callback race** — Wizard callback dostane `&Arc<Mutex<AppShared>>`, ne `&mut WorkspaceState`. Nastavení `pending_open_choice` vyžaduje buď: (a) přístup k ws přes wizard_state, nebo (b) nový AppAction (např. `OpenWithChoice(PathBuf)`) zpracovaný na workspace úrovni. Varianta (b) je čistší — wizard nemá přímý přístup k ws.
- **Escape v modalu** — egui `Modal` zavře na Escape a click mimo. Musí to znamenat Cancel (ne "přeskočit dialog a otevřít"). Ošetřit `should_close()` → čistit pending stav.
- **Duplicitní modal při rychlém klikání** — Pokud `pending_open_choice` je `Some` a uživatel klikne znovu na recent/open project, nový výběr by měl nahradit starý (ne akumulovat). `pending_open_choice = Some(new_path)` přirozeně přepisuje.
- **Unsaved guard cancel → čistit pending_open_choice** — Pokud uživatel v guard dialogu zruší (Cancel), `pending_open_choice` se musí vyčistit, jinak modal vyskočí znovu.
- **Already-open check** — `open_in_new_window()` kontroluje, jestli projekt není už otevřený (řádek 301-314 `app/mod.rs`). Pro "stávající okno" tato kontrola není relevantní (měníme workspace na místě), ale pro "nové okno" volbu se použije existující logika automaticky.

## Open Risks

- **`SwitchProject` guard mode interakce s global close guard** — `start_global_close_guard` nastaví `pending_global_close` a workspace `pending_close_flow`. Pokud uživatel spustí "přepnout projekt" (SwitchProject guard) a zároveň klikne na X okna, dvě guard flow by kolidovaly. Mitigace: blokovat open choice modal pokud `pending_close_flow.is_some()`.
- **Secondary viewport wizard** — Wizard se renderuje přes `render_dialogs()` v `workspace/mod.rs` i pro secondary viewporty. Callback potřebuje buď: (a) přepojit na `pending_open_choice` (ale wizard nemá &mut ws v callbacku), nebo (b) nový signál přes shared actions. Potřeba ověřit cestu dat při planning.
- **Session persistence po reinitu** — Root workspace reinit (`app/mod.rs:879-881`) volá `self.push_recent()` a `self.save_session()`. Secondary viewport reinit to řeší přes `AppAction::AddRecent`. Obě cesty fungují — jen ověřit, že se `save_session` zavolá i v secondary viewport scénáři.

## Candidate Requirements

Na základě výzkumu navrhuji tyto candidate requirements (k potvrzení v planning):

| ID | Candidate | Class | Notes |
|----|-----------|-------|-------|
| R037 | Modal dialog s volbou "Nové okno (výchozí) / Stávající okno / Zrušit" po výběru složky v "Otevřít projekt" | core-capability | Primární deliverable |
| R038 | Stejný modal po vytvoření projektu v "Nový projekt" wizardu | core-capability | Wizard callback přesměrování |
| R039 | Stejný modal po kliknutí na položku v "Nedávné projekty" | core-capability | Recent handler přesměrování |
| R040 | Unsaved changes guard při volbě "stávající okno" (Save / Discard / Cancel) | failure-visibility | Rozšíření stávajícího guard flow |
| R041 | Workspace reinicializace: cleanup + nový `init_workspace()` | core-capability | Existující `open_here_path` pattern |
| R042 | i18n pro nový dialog ve všech 5 jazycích (cs, en, sk, de, ru) | launchability | ~6-8 nových klíčů × 5 jazyků |
| R043 | Terminály, watchers a background procesy se korektně ukončí při přepnutí | operational | Pokryt Rust Drop — spíš verifikační requirement |

**Nenavržené (advisory):**
- "Zapamatovat poslední volbu" — explicitně out of scope v kontextu
- "Default chování v settings" — explicitně out of scope
- "Keyboard shortcut pro volbu v modalu" — nice-to-have, ale Enter pro výchozí (Nové okno) je standard egui behavior

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | none | No egui-specific skills found |
| Rust | `jeffallan/claude-skills@rust-engineer` (1.1K installs) | available — generic Rust, not egui-specific. Not needed for this scope. |

## Sources

- Codebase analysis: `src/app/mod.rs` (reinit pattern řádky 431-451, 863-881), `src/app/ui/workspace/mod.rs` (open_here_path flow), `src/app/ui/workspace/menubar/mod.rs` (menu action handlers), `src/app/ui/workspace/state/mod.rs` (PendingCloseFlow), `src/app/ui/dialogs/confirm.rs` (unsaved guard dialog), `src/app/ui/widgets/modal.rs` (modal patterns), `src/app/ui/terminal/instance/mod.rs` (Terminal Drop cleanup)
