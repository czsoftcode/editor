# Phase 35 Research: Trash Foundation + Async Safety

**Phase:** 35-trash-foundation-async-safety  
**Date:** 2026-03-11  
**Scope guard:** pouze foundation pro `.polycredo/trash` a non-blocking execution model pro delete path. Bez restore UI a bez retention policy.

## Research Summary

Pro phase 35 je klíčové dodat minimální bezpečný základ:
- deterministické vytvoření `.polycredo/trash` (on-demand),
- fail-closed chování bez hard-delete fallbacku,
- přesun těžších FS operací mimo UI thread,
- error surfacing do stávající toast pipeline.

Největší riziko je regresní UX z blokujícího I/O v `file_tree` delete dialogu a skrytá data-loss cesta přes fallback na `remove_*` při failure create/move.

## Existing Code Signals

- `src/app/ui/file_tree/dialogs.rs` nyní používá synchronní `std::fs::remove_dir_all/remove_file` přímo v potvrzení delete.
- `src/app/ui/background.rs::spawn_task` je zavedený pattern pro non-blocking práci.
- `src/app/project_config.rs` už centralizuje `.polycredo` cesty (`project_config_dir`), vhodné pro rozšíření o `trash_dir` helper.
- `.polycredo` je již interní namespace (schovaný ve file tree, filtrovaný watcherem), takže trash uvnitř `.polycredo` je kompatibilní se stávající architekturou.

## Implementation Direction (phase-scoped)

1. Přidat path helper pro `.polycredo/trash` do `project_config` vrstvy.
2. Přesunout delete tok na async/background execution.
3. V delete toku nejdřív zajistit existenci trash (create_dir_all on-demand), teprve pak provést move.
4. Při chybě create/move zastavit operaci a vrátit konkrétní toast error.
5. Žádný restore UI v této fázi (phase 37).

## Pitfalls To Avoid

- Hard-delete fallback po selhání create/move (porušení bezpečnostního cíle).
- Blokující I/O na UI vlákně během potvrzení delete.
- Nejednoznačné error texty bez příčiny/akce.
- Automatický retry smyčka bez explicitní nové user akce.

## Validation Architecture

### 1) Validation Layers
- Layer A: cílené testy/delete-path assertions (fail-closed + path setup).
- Layer B: compile + quality gate (`cargo check`, `./check.sh`).
- Layer C: planning evidence (summary/verification mapping na TRASH-03 a RELIAB-01).

### 2) Requirement Mapping
- TRASH-03 <- helper + on-demand create + create failure behavior.
- RELIAB-01 <- async execution path + absence UI blocking toku v delete flow.

### 3) Execution Order
1. Foundation helpery + async wiring.
2. Delete path rewrite na create+move fail-closed.
3. Testy/verification evidence.
4. Full quality gate.

### 4) Evidence Contract
Každý task musí mít:
- konkrétní soubory,
- automatizovaný verify command,
- done statement navázaný na requirement ID.

## Suggested Plan Split

- **Plan 35-01 (Wave 1):** path helper + async foundation wiring + fail-closed create behavior.
- **Plan 35-02 (Wave 2):** delete flow finalize, test coverage, quality gate evidence.

## Out of Scope Enforcement

- Restore workflow a konfliktní restore policy.
- Trash browse UI.
- Retention cleanup automatizace.
- OS recycle bin bridge.
