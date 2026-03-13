# Phase 37 Research: Trash Preview + Restore MVP (FORCE Re-Research)

**Phase:** 37-trash-preview-restore-mvp  
**Date:** 2026-03-12  
**Mode:** FORCE re-research  
**Scope guard:** pouze `TRASHUI-01`, `RESTORE-01`, `RESTORE-02`, `RESTORE-03`.

## Context Anchoring (37-CONTEXT.md)

Tento research explicitně navazuje na rozhodnutí v `.planning/phases/37-trash-preview-restore-mvp/37-CONTEXT.md`:
- Trash preview je modal, vstup přes menu/command, ne přes file-tree uzel.
- Výchozí řazení preview: nejnovější položky nahoře.
- Restore je single-item (MVP), s nedestruktivní conflict policy.
- Při konfliktu cíle není povolen tichý overwrite; povoleno pouze `Restore as copy` nebo `Cancel`.
- Po úspěšném restore proběhne okamžitý lokální refresh UI + watcher dorovnání.
- Obnovený soubor se v MVP automaticky neotvírá do tabu.

Tento rámec je konzistentní s `.planning/REQUIREMENTS.md` (pending: TRASHUI-01, RESTORE-01/02/03) i se stavem v `.planning/STATE.md` (po dokončení phase 36, připraveno plánovat phase 37).

## Requirement Coverage (Targeted)

| Requirement | Co musí být implementačně a validačně prokázáno |
|---|---|
| `TRASHUI-01` | Modal preview seznamu `.polycredo/trash` + akce restore vybrané položky |
| `RESTORE-01` | Obnova jedné položky z trash na původní cestu |
| `RESTORE-02` | Konflikt cílové cesty řešen nedestruktivně, bez silent overwrite |
| `RESTORE-03` | Po restore konzistentní aktualizace stromu/tabů bez restartu |

## Implementation Architecture

### Engine boundary (`src/app/trash.rs`)

Doporučené rozšíření stávajícího trash engine tak, aby UI pouze orchestruje:
- `list_trash_entries(project_root) -> Result<Vec<TrashListEntry>, TrashError>`
- `restore_from_trash(project_root, entry_id_or_path, policy) -> Result<TrashRestoreOutcome, TrashError>`

Důvod: stejné místo drží delete kontrakt a fail-closed styl chyb, takže restore nebude fragmentovaný do UI vrstev.

### Metadata contract (restore mapování)

Pro spolehlivé mapování trash položky na původní cestu je potřeba per-entry metadata persistence:
- při delete ukládat metadata sidecar (`*.meta.json`) vedle položky v trash,
- preview čte list z trash + sidecar metadat,
- restore je řízen výhradně validními metadaty (ne heuristikou názvu souboru).

Fail-closed pravidlo: pokud metadata chybí nebo jsou nevalidní, položka je viditelná jako neobnovitelná + UI nabídne srozumitelnou chybu/toast.

### Async UI orchestration

Zachovat existující neblokující pattern:
- modal triggerne restore,
- restore běží mimo UI vlákno (`spawn_task` + `mpsc`),
- UI poll (`try_recv`) zpracuje výsledek a nastaví pending signály pro reload/toast.

To přímo drží `RELIAB-01` a navazuje na stávající file-tree flow.

## Integration Points

- `src/app/trash.rs`
- list API, restore API, conflict copy resolver, error kategorizace.

- `src/app/ui/file_tree/dialogs.rs`
- modal trash preview, modal conflict volby, mapování restore chyb na toast.

- `src/app/ui/file_tree/mod.rs`
- UI state pro preview + restore channel polling, pending reload signal.

- `src/app/ui/workspace/menubar/mod.rs`
- menu action pro otevření trash preview.

- `src/app/ui/widgets/command_palette.rs`
- command entrypoint pro trash preview (parita s menu).

- `src/app/ui/panels.rs`
- finalizace UI refresh kontraktu po úspěchu restore (reload/expand/highlight, bez auto-open tabu).

- `locales/*/ui.ftl`
- nové i18n klíče pro preview, conflict dialog, success/error toasty.

## Risk Register

1. Metadata drift mezi trash položkou a sidecar souborem.
Mitigace: fail-closed restore, explicitní UI stav „nelze obnovit“, žádný fallback overwrite.

2. Race mezi lokálním refresh a watcher eventy.
Mitigace: jeden autoritativní pending-refresh hook po restore; watcher pouze eventual consistency.

3. Kolize při generování copy názvu v konfliktu.
Mitigace: deterministický suffix algoritmus + bounded pokusy (např. 0..1000) + jasná chyba při vyčerpání.

4. i18n drift (chybějící klíče v některém jazyce).
Mitigace: doplnění všech jazyků v jednom patchi + parity gate z `./check.sh`.

5. Překvapivé FS chování (permissions / rename edge-cases).
Mitigace: explicitní error kategorizace a toast instrukce, bez destruktivního fallbacku.

## Test Strategy

### Engine tests
- list vrací očekávané položky a řazení (nejnovější první),
- restore happy-path vrací očekávanou cílovou cestu,
- restore konflikt nikdy nepřepíše existující soubor,
- restore při chybějícím parent adresáři vytvoří parent,
- metadata missing/corrupt vrací deterministickou chybu.

### UI/state tests
- preview action otevře modal stav,
- restore result přes polling nastaví reload/expand signál,
- disconnect/error větev restore kanálu propaguje toast.

### i18n/regression gates
- nové klíče ve všech `cs/en/de/ru/sk`,
- minimální gate: `cargo check` + `./check.sh`.

## Validation Architecture

Nyquist validační architektura pro phase 37 bude evidence-first a requirement-backward:

- Validation Unit `TRASHUI-01`
- Hook: preview modal entrypoint + restore trigger dostupný z preview.
- Evidence: focused UI/state test + manuální screenshot/flow poznámka.

- Validation Unit `RESTORE-01`
- Hook: engine restore jedné položky na origin path.
- Evidence: unit test restore happy-path + sidecar cleanup/consistency check.

- Validation Unit `RESTORE-02`
- Hook: konflikt cílové cesty.
- Evidence: test, že silent overwrite nenastane; povolen pouze copy/cancel flow.

- Validation Unit `RESTORE-03`
- Hook: post-restore UI synchronizace.
- Evidence: test pending reload/expand/highlight kontraktu + no-auto-open tabu.

- Final quality gates
- `cargo check`
- `./check.sh`
- Traceability tabulka requirement -> test/evidence v následném `37-VERIFICATION.md`.

## Wave Proposal

### Wave 1: Engine + metadata contract
- Rozšířit `trash.rs` o list/restore API.
- Zavést sidecar metadata contract a conflict copy resolver.
- Přidat jednotkové testy engine vrstvy.

### Wave 2: Preview UI + entrypointy
- Přidat menu + command trigger pro trash preview.
- Implementovat preview modal (seznam + filtr + výběr položky).
- Napojit restore trigger na async job pipeline.

### Wave 3: Conflict UX + post-restore sync
- Implementovat conflict modal (`Restore as copy` / `Cancel`).
- Dotáhnout reload/expand/highlight kontrakt bez auto-open tabu.
- Doplnit i18n klíče a toast texty.

### Wave 4: Nyquist validation + gates
- Dodat requirement-focused testy a evidence mapování.
- Spustit `cargo check` a `./check.sh`.
- Uzavřít traceability pro TRASHUI-01 + RESTORE-01/02/03.

## Open Decisions To Lock In Plan

- Finální formát sidecar názvu (`*.meta.json`) a lifecycle po restore.
- Přesný suffix pattern pro „restore as copy“ (deterministický, čitelný).
- Finální wording conflict/success toastů pro všechny jazyky.

## RESEARCH COMPLETE
