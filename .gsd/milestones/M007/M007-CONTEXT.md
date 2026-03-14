# M007: Dialog Otevření Projektu — Stávající vs Nové Okno — Context

**Gathered:** 2026-03-14
**Status:** Queued — pending auto-mode execution.

## Project Description

Přidání modálního dialogu s volbou "Otevřít ve stávajícím okně" / "Otevřít v novém okně" po výběru složky (Otevřít projekt), po vytvoření projektu (Nový projekt) a po kliknutí na položku v Nedávných projektech. Ve stávajícím okně se zavře aktuální projekt (s unsaved changes guardem) a otevře vybraný/nový.

## Why This Milestone

Aktuálně mají projektové akce hardcoded chování:
- **"Otevřít projekt"** (`menu-project-open`) vždy otevírá v novém okně (`in_new_window = true` v `spawn_task` callbacku).
- **"Nový projekt"** wizard vždy volá `AppAction::OpenInNewWindow(path)`.
- **"Nedávné projekty"** (submenu) vždy volají `AppAction::OpenInNewWindow(path)`.
- **"Otevřít složku"** (`menu-file-open-folder`) vždy otevírá ve stávajícím okně (`in_new_window = false`).

Uživatel nemá volbu. Pokud chce otevřít projekt ve stávajícím okně (nahradit aktuální workspace), musí to udělat přes File → Open Folder, což je neintuitivní. A naopak, pokud chce nedávný projekt v novém okně, nemá alternativu (i když to je current behavior). Žádná z akcí se neptá.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Kliknout na "Otevřít projekt" → vybrat složku v file dialogu → zobrazí se modal "Otevřít v novém okně (výchozí) | Otevřít ve stávajícím okně" → vybere si
- Kliknout na "Nový projekt" → projít wizardem → po vytvoření se zobrazí stejný modal → vybere si
- Kliknout na položku v "Nedávné projekty" → zobrazí se stejný modal → vybere si
- Při volbě "stávající okno" se zobrazí unsaved changes guard (pokud jsou neuložené změny) s volbou Uložit / Zahodit / Zrušit
- Po potvrzení se aktuální projekt zavře (terminály, watchers, local history) a otevře se nový projekt ve stejném okně

### Entry point / environment

- Entry point: menu Projekt → Otevřít / Nový / Nedávné
- Environment: desktop editor (eframe/egui), local-first, single-process multi-window
- Live dependencies involved: none — vše je lokální

## Completion Class

- Contract complete means: po každé ze tří akcí se zobrazí modal s volbou; volba "nové okno" vytvoří nový viewport jako dnes; volba "stávající okno" provede workspace reinicializaci; unsaved guard se zobrazí při neuložených změnách; unit testy pokrývají guard flow a reinicializaci.
- Integration complete means: celý tok funguje end-to-end — výběr složky → modal → volba "stávající" → guard (pokud potřeba) → starý workspace cleanup → nový workspace init → editor ukazuje nový projekt; stejný tok pro wizard a recent.
- Operational complete means: terminály starého projektu se korektně zavřou; watcher se přepojí na nový adresář; local history se reinicializuje; session persistence se aktualizuje.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Menu → Otevřít projekt → vybrat složku → modal se zobrazí → klik "Nové okno" → nový viewport vznikne (jako dnes). Klik "Stávající okno" → starý projekt se zavře a nový se otevře ve stejném okně.
- Menu → Nový projekt → wizard → vytvořit → modal → "Stávající okno" → starý workspace se nahradí novým.
- Menu → Nedávné → klik na projekt → modal → volba → správné chování.
- Při "stávajícím okně" s neuloženými změnami → unsaved changes guard → Uložit → soubory se uloží → projekt se přepne. Guard → Zrušit → nic se nestane.
- Terminály (build, AI) starého projektu se korektně ukončí při přepnutí.
- `cargo check` + `./check.sh` projde čistě.

## Risks and Unknowns

- **Workspace reinicializace** — Aktuálně se workspace vytváří přes `init_workspace()` jednou při startu. Reinicializace ve stávajícím okně vyžaduje: ukončení terminálů (PTY cleanup), odpojení watcherů, reset editor stavu, nové `init_workspace()` volání, a přepojení na `root_ws` nebo `SecondaryWorkspace.state`. Není jasné, jestli `init_workspace()` zvládne reinit na existující Mutex<WorkspaceState>, nebo bude potřeba nový pattern.
- **PTY cleanup při přepnutí** — Terminálové backendy (build, AI chaty) drží PTY file descriptory. Při přepnutí projektu se musí gracefully ukončit, jinak zůstanou zombie procesy. Stávající close path (`CloseWorkspace` action) zavírá celý viewport — tady zavíráme jen workspace obsah.
- **Watcher reconnect** — `FileWatcher` je napojený na `root_path`. Přepnutí projektu vyžaduje nový watcher na nový adresář. Starý musí být zrušen.
- **Unsaved changes guard interakce** — Existující `pending_close_flow` je navržen pro zavření tabu/okna. Pro "přepnutí projektu" bude potřeba nový guard flow (nebo rozšíření stávajícího), který po úspěchu pokračuje reinicializací místo zavření.
- **Borrow checker** — `WorkspaceState` je za `Arc<Mutex<>>` v secondary viewports. Reinicializace vyžaduje buď kompletní replace obsahu mutexu, nebo drop + nový mutex. Pattern závisí na tom, jestli reinit jde na root_ws (primární) nebo secondary.

## Existing Codebase / Prior Art

- `src/app/ui/workspace/menubar/mod.rs` — `process_menu_actions()` zpracovává `open_project`, `new_project`, `open_recent`. `open_project` spouští `FileDialog` s `in_new_window = true`. `open_recent` pushuje `AppAction::OpenInNewWindow`. Vrací `open_here_path: Option<PathBuf>` pro otevření ve stávajícím okně.
- `src/app/ui/workspace/menubar/project.rs` — Menu rendering: "Otevřít projekt", "Nový projekt", "Koš", "Nedávné projekty" submenu.
- `src/app/ui/workspace/modal_dialogs.rs` — New project wizard callback volá `AppAction::OpenInNewWindow(path)`. Dialog rendering pipeline.
- `src/app/mod.rs` — `open_in_new_window()` vytváří `SecondaryWorkspace` s novým `init_workspace()`. `process_actions()` zpracovává `AppAction` enum. Multi-viewport architektura s `root_ws` a `secondary` Vec.
- `src/app/types.rs` — `AppAction` enum: `OpenInNewWindow(PathBuf)`, `CloseWorkspace(ViewportId)`, `AddRecent(PathBuf)`, `QuitAll`.
- `src/app/ui/workspace/state/init.rs` — `init_workspace()` inicializuje `WorkspaceState` z cesty, nastavuje terminály, watchers, editor stav.
- `src/app/ui/workspace/mod.rs` — `render_workspace()` s `open_here_path` pattern — existující mechanismus pro otevření jiného projektu ve stávajícím okně, ale aktuálně se používá jen pro `open_folder` (File → Open Folder).
- `src/app/ui/background.rs` — `process_unsaved_close_guard_dialog()` — stávající unsaved changes guard pro zavření tabů/oken.
- `src/app/ui/widgets/modal.rs` — `show_modal()` pattern pro jednoduché confirm dialogy.
- `src/app/ui/dialogs/` — `show_project_wizard()` pro nový projekt.
- `locales/*/menu.ftl` — Existující i18n klíče pro menu položky.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- Nový scope — zavádí nové requirements pro dialog volby okna při otevření/vytvoření projektu.
- Částečně relevantní: V-3 (Udržet file dialog asynchronní) — file dialog už je asynchronní přes `spawn_task`, modal volby se zobrazí až PO návratu výsledku, neblokuje UI.

## Scope

### In Scope

- Modal dialog s volbou "Otevřít v novém okně" (výchozí/zvýrazněný) / "Otevřít ve stávajícím okně" po:
  - Výběru složky v "Otevřít projekt"
  - Vytvoření projektu v "Nový projekt" wizardu
  - Kliknutí na položku v "Nedávné projekty"
- Unsaved changes guard při volbě "stávající okno" (Uložit / Zahodit / Zrušit)
- Workspace reinicializace ve stávajícím okně: cleanup terminálů, watcherů, editor stavu + nový `init_workspace()`
- i18n pro nový dialog ve všech 5 jazycích (cs, en, sk, de, ru)

### Out of Scope / Non-Goals

- Přidání "Otevřít složku" (File → Open Folder) do tohoto flow — ten zůstává jako dnes (vždy stávající okno, bez dialogu)
- Drag & drop složky na editor jako způsob otevření projektu
- Zapamatování poslední volby (vždy se ptát znovu)
- Settings toggle pro defaultní chování (vždy nové okno / vždy stávající / vždy se ptát)
- Tab-based projekty (více projektů v jednom okně jako taby)

## Technical Constraints

- `cargo check` + `./check.sh` musí projít po každé slice
- Žádné nové runtime závislosti
- Neblokovat UI vlákno — file dialog zůstává asynchronní přes `spawn_task`
- Modal volby se zobrazí SYNCHRONNĚ po návratu výsledku z file dialogu (žádný async)
- Zachovat existující multi-viewport architekturu (root_ws + secondary)
- Workspace reinicializace musí korektně ukončit PTY procesy (build + AI terminály)
- Stávající `open_folder` (File → Open Folder) se nemění — zůstává bez dialogu

## Integration Points

- `src/app/ui/workspace/menubar/mod.rs` — Přepojení `open_project`, `open_recent` na nový dialog flow místo přímého `AppAction::OpenInNewWindow`. Rozšíření `process_menu_actions()` o pending stav pro dialog.
- `src/app/ui/workspace/modal_dialogs.rs` — Nový modal pro volbu okna. Wizard callback přepojit na pending stav místo přímého `OpenInNewWindow`.
- `src/app/ui/workspace/state/mod.rs` — Nový stav pro pending "kam otevřít" dialog (cesta + zdroj akce).
- `src/app/ui/workspace/mod.rs` — `open_here_path` logika pro workspace reinicializaci (existující mechanismus, ale potřebuje rozšíření o cleanup).
- `src/app/mod.rs` — Případné rozšíření pro reinicializaci root_ws (pokud se přepíná primární okno).
- `src/app/ui/background.rs` — Napojení na unsaved changes guard flow.
- `locales/*/ui.ftl` — Nové i18n klíče pro dialog.

## Open Questions

- **Reinicializace root_ws vs secondary** — Pokud uživatel přepíná projekt v primárním okně (`root_ws`), je potřeba nahradit celý `WorkspaceState` v `Option<WorkspaceState>` na `App` struct úrovni. V secondary viewport je to replace obsahu `Arc<Mutex<WorkspaceState>>`. Detaily závisí na tom, jak `render_workspace()` caller zpracovává `open_here_path` — ověřit při planning.
- **Cleanup scope** — Co přesně se musí vyčistit při přepnutí: terminály (PTY kill), watcher (stop), file tree state, editor tabs, local history, search panel, build errors, LSP connection. Potřeba projít `WorkspaceState` fieldy a rozhodnout co se reinit a co se drop. Nebo jednodušeji: celý `WorkspaceState` nahradit novým z `init_workspace()`.
- **Stav dialogu pro wizard** — Wizard produkuje cestu asynchronně (uživatel prochází formuláři). Modal volby okna se musí zobrazit AŽ PO úspěšném vytvoření projektu, ne před wizardem. Wizard callback aktuálně přímo pushuje `OpenInNewWindow` — bude potřeba přesměrovat na pending stav.
