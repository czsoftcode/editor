# M007: Dialog Otevření Projektu — Stávající vs Nové Okno

**Vision:** Po výběru složky / vytvoření projektu / kliknutí na nedávný projekt se zobrazí modal s volbou "Nové okno" / "Stávající okno". Volba "stávající" provede unsaved guard a workspace reinicializaci přes existující `open_here_path` pattern.

## Success Criteria

- Menu → Otevřít projekt → vybrat složku → modal "Nové okno (výchozí) / Stávající okno / Zrušit" se zobrazí
- Menu → Nový projekt → wizard → vytvořit → stejný modal se zobrazí
- Menu → Nedávné → klik na projekt → stejný modal se zobrazí
- Volba "Nové okno" → nový viewport vznikne (jako dnes)
- Volba "Stávající okno" s neuloženými změnami → unsaved changes guard (Save / Discard / Cancel) se zobrazí
- Volba "Stávající okno" bez neuložených změn → starý workspace se nahradí novým ve stejném okně
- Guard → Cancel → nic se nestane, modal zmizí
- Guard → Save → soubory se uloží → projekt se přepne
- Guard → Discard → projekt se přepne bez uložení
- Terminály starého projektu se korektně ukončí při přepnutí (Rust Drop na WorkspaceState)
- `cargo check` + `./check.sh` projde čistě
- i18n klíče pro nový dialog ve všech 5 jazycích (cs, en, sk, de, ru)

## Key Risks / Unknowns

- **Guard flow rozšíření** — Stávající `PendingCloseFlow` je navržen pro zavření workspace, ne pro "přepni a pokračuj reinitem". Po dokončení guard flow (save/discard) musí následovat reinicializace místo close. Potřeba nového `PendingCloseMode::SwitchProject(PathBuf)` a post-guard signálu.
- **Wizard callback data path** — Wizard callback má `&Arc<Mutex<AppShared>>`, ne `&mut WorkspaceState`. Nastavení `pending_open_choice` vyžaduje nový `AppAction` variant (např. `OpenWithChoice(PathBuf)`), zpracovaný na workspace úrovni.

## Proof Strategy

- Guard flow rozšíření → retire in S01 tím, že volba "stávající okno" s dirty tabs spustí guard, Save uloží a přepne, Cancel vrátí zpět
- Wizard callback → retire in S01 tím, že po vytvoření projektu ve wizardu se zobrazí modal (ne přímý `OpenInNewWindow`)

## Verification Classes

- Contract verification: `cargo check` + `./check.sh` (229+ testů), manuální ověření flow přes UI
- Integration verification: celý tok menu → modal → guard → reinit v živém editoru
- Operational verification: terminály starého projektu se ukončí (Drop cleanup), watcher se přepojí
- UAT / human verification: vizuální ověření modalu a guard interakce v editoru

## Milestone Definition of Done

This milestone is complete only when all are true:

- [x] Modal dialog se zobrazí po všech třech akcích (open project, new project wizard, recent)
- [x] "Nové okno" vytvoří nový viewport
- [x] "Stávající okno" provede workspace reinicializaci (starý ws se dropne, nový init_workspace)
- [x] Unsaved changes guard se zobrazí při dirty tabs a správně reaguje na Save/Discard/Cancel
- [x] i18n klíče existují ve všech 5 jazycích
- [x] `cargo check` + `./check.sh` projde čistě
- [x] Escape/klik mimo modal = Cancel (vyčistí pending stav)

## Requirement Coverage

- Covers: R037, R038, R039, R040, R041, R042, R043
- Partially covers: none
- Leaves for later: none
- Orphan risks: none

*Poznámka: R037–R043 jsou nové requirements definované v M007-RESEARCH.md. Budou přidány do REQUIREMENTS.md jako active při start execution, validated po dokončení.*

## Slices

- [x] **S01: Modal volby okna s guard flow a workspace reinicializací** `risk:high` `depends:[]`
  > After this: po výběru složky / vytvoření projektu / kliknutí na nedávný projekt se zobrazí modal s volbou; "Nové okno" otevře nový viewport; "Stávající okno" spustí unsaved guard (pokud dirty), po guard dokončení starý workspace se nahradí novým; i18n kompletní

## Boundary Map

### S01 (single slice — no downstream)

Produces:
- `pending_open_choice: Option<PathBuf>` stav na `WorkspaceState` pro pending modal
- `PendingCloseMode::SwitchProject(PathBuf)` variant pro guard flow s post-guard reinicializací
- `AppAction::OpenWithChoice(PathBuf)` pro wizard callback data path
- `show_open_choice_modal()` helper pro 3-tlačítkový modal
- i18n klíče `open-choice-*` v `locales/*/ui.ftl`

Consumes:
- Existující `open_here_path` pattern v `app/mod.rs` (root: 863-881, secondary: 431-451)
- Existující `PendingCloseFlow` + `process_unsaved_close_guard_dialog()` v `workspace/mod.rs`
- Existující `show_modal()` pattern + `egui::Modal` pro custom layout
- Existující `spawn_task` + `folder_pick_rx` async file dialog pattern
