# Phase 36 Research: Safe Move-to-Trash Engine

**Phase:** 36-safe-move-to-trash-engine  
**Date:** 2026-03-12  
**Scope guard:** pouze delete workflow (TRASH-01, TRASH-02, TRASH-04, RELIAB-02). Bez restore UI/flow a bez retention cleanup policy.

## Research Summary

Phase 35 uz dodala funkcni foundation (`ensure_trash_dir`, `move_path_to_trash`, async delete dispatch mimo UI vlakno). Pro dobre naplanovani phase 36 je potreba navrhnout minimalni rozsireni, ktere:
- potvrdi robustni chovani pro soubory i adresare vcetne kolizi,
- dotahne fail-safe a toast-first error surfacing i pro edge cesty,
- zablokuje mazani interniho `.polycredo/trash` kontraktem v engine vrstve,
- neprida restore capability ani jine mimo-scope API.

Nejvetsi planovaci riziko je, ze cast kontraktu zustane pouze implicitni ("funguje nahodou") misto explicitnich guardu/testu pro phase 36.

## Current Baseline (What Already Exists)

- `src/app/trash.rs`
- `move_path_to_trash(project_root, source_path)` kanonizuje cesty, vynucuje `source starts_with project`, vytvori `.polycredo/trash` on-demand a dela `std::fs::rename` (fail-closed, bez hard-delete fallbacku).
- Kolize jsou resene suffixem `file.trash-{deleted_at}` + `-{attempt}` (max 1000 pokusu), se zachovanim relativni struktury podadresaru.
- Pro adresar delete se presouva cely strom atomicky na urovni `rename`; pri chybe vraci error a puvodni data zustavaji.

- `src/app/ui/file_tree/dialogs.rs`
- Confirm delete uz bezi pres `spawn_task` (neblokuje UI) a vysledek jde pres `DeleteJobResult`.
- Chyby jdou jako `DeleteJobResult::Error("trash move failed: ...")`.

- `src/app/ui/file_tree/mod.rs` + `src/app/ui/panels.rs`
- `pending_error` se zveda do toast pipeline (`ws.toasts.push(Toast::error(err))`).

- Skryti interniho namespace
- `.polycredo` je ignorovany ve file tree (`IGNORED_DIRS`) i project watcheru.

## Gaps To Cover In Phase 36 Plan

1. Explicitni blokace mazani `.polycredo/trash` v engine vrstve.
- V kontextu faze je rozhodnuto, ze pokus o smazani trash adresare se ma blokovat a reportovat toastem.
- Aktualni kod spoleha spis na to, ze `.polycredo` neni videt ve file tree; to je slaba obrana (ne explicitni kontrakt).

2. Requirement-level dukaz pro TRASH-01/TRASH-02.
- Funkcne uz move-to-trash existuje, ale phase 36 potrebuje cilene testy mapovane na requirementy (soubor/adresar + kolize + fail-safe).

3. RELIAB-02 pokryti pro I/O chyby.
- Je potreba mit jasne dokazatelne, ze chyby z delete toku vzdy dopadnou do toast pipeline (nejen "typicke" chyby).
- Plan ma explicitne rozhodnout, co delat pri abnormalnich stavech (napr. kanal odpojen/paniky workeru) tak, aby uzivatel nezustal bez feedbacku.

4. UX kontrakt pro chybove hlasky.
- Kontekst chce kratkou, konkretni pricinu + doporuceni dalsiho kroku, a pri sekvenci selhani preferenci souhrnneho signalu (bez spamovani).
- V phase 36 je realisticke drzet minimalni implementaci: jednotny kvalitni error text pro jednotlive delete akce; agregace vice chyb muze byt jen pokud nevznikne architektonicka slozitost.

## Integration Points You Need For Planning

- Primarni editace:
- `src/app/trash.rs` (guardy, kolizni/validacni kontrakt, chyby)
- `src/app/ui/file_tree/dialogs.rs` (mapovani engine error -> UI-friendly message)
- `src/app/ui/file_tree/mod.rs` (spolehlive vyzvednuti async vysledku)

- Testy:
- rozsirit `tests/phase35_*` style o `tests/phase36_*` (contract testy nad zdrojaky + pripadne behavior testy helperu pokud jsou vhodne bez velkeho refactoru)

- Lokalizace:
- pokud pribudou nove user-facing chyby, doplnit `locales/{cs,en,de,ru,sk}/ui.ftl` klice konzistentne.

## Recommended Plan Shape (Minimal, Phase-Scoped)

1. Plan A: Trash engine safety contracts
- Pridat explicitni blokaci pokusu smazat `.polycredo/trash` (a pripadne jeho predky podle rozhodnuteho pravidla).
- Udrzet fail-closed semantiku a beze zmeny "no hard-delete fallback".

2. Plan B: UI propagation + error wording
- Zarucit, ze vsechny delete I/O fail stavy skonci v toastu.
- Udrzet modal-close behavior po chybe a bez auto-retry.

3. Plan C: Requirement-focused verification
- Dodat cilenou test evidence pro TRASH-01, TRASH-02, TRASH-04, RELIAB-02.
- Dokoncit quality gate: `cargo check` + `./check.sh`.

## Pitfalls To Avoid

- Zavleceni restore API/symbolu do phase 36.
- "Silent" fallback na hard-delete pri rename failure.
- Zavislost na tom, ze `.polycredo` je skryta v UI (misto explicitniho guardu v engine).
- Blokujici I/O v UI event handleru.
- Rozsahovy drift do watcher performance tematu (RELIAB-03 je az phase 38).

## Validation Architecture

### 1) Validation Layers
- Layer A (Contract): trash engine guardy + fail-closed path + kolizni naming policy.
- Layer B (UI Reliability): async delete flow + error->toast propagation bez ztraty signalu.
- Layer C (Quality Gate): `cargo check` a `./check.sh` pass.

### 2) Requirement Mapping
- TRASH-01: soubor jde do `.polycredo/trash`, bez hard-delete fallbacku.
- TRASH-02: adresar jde do `.polycredo/trash` vcetne stromu; pri chybe zustava puvodni stav.
- TRASH-04: pri move failure nedojde ke ztrate dat a uzivatel vidi toast chybu.
- RELIAB-02: delete I/O chyby jsou propagovane do UI (toast-first).

### 3) Evidence Contract
Kazdy plan task by mel vyrobit:
- konkretni diff v uvedenych souborech,
- test nebo aspon regression guard odpovidajici requirement ID,
- zaznam verify prikazu a PASS/FAIL vysledku.

### 4) Nyquist Focus
- Overit nejen happy-path, ale i failure-path (permissions, invalid target state, blocked trash delete).
- Overit, ze ochrana dat je priorita: pri chybe nema nastat odstraneni puvodni polozky.
- Overit, ze UI dostane chybu deterministicky (ne pouze best-effort log).

## Open Decisions To Set During Planning

- Jak presne definovat "blokaci mazani `.polycredo/trash`": jen root trash adresar, nebo i jakykoli path pod nim.
- Zda delat explicitni handling `mpsc::TryRecvError::Disconnected` v `FileTree` poll smycce pro robustnejsi RELIAB-02.
- Jestli phase 36 zavede nove i18n klice pro trash-specific chyby, nebo zustane u stavajiciho obecneho formatu s duvodem.

## RESEARCH COMPLETE
