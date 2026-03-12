# Phase 38 Research: Watcher Stability + Verification

**Phase:** 38-watcher-stability-verification  
**Date:** 2026-03-12  
**Scope guard:** pouze `RELIAB-03` (watcher/event handling po delete/restore). Bez nových user feature mimo stabilizaci watcher pipeline.

## Kontext a cílový problém

`RELIAB-03` je jediné otevřené v1 requirement. Kontext fáze 38 už zamknul cílovou politiku:
- dedupe po klíči `path + kind`,
- batching okno 100-150 ms,
- při kolizi na stejné cestě má prioritu `remove` nad `modify/create`,
- při event špičce/overflow bezpečný fallback na jeden full reload.

Aktuální flow (`src/watcher.rs` -> `src/app/ui/background.rs`) sice dávkuje eventy, ale bez plně explicitní normalizace finálního stavu může delete/restore burst vyvolat nadbytečné index/reload operace a viditelný UI lag.

## Architektonické závěry pro plán

### 1) Hranice odpovědnosti

- `src/watcher.rs` musí být autorita pro ingest + normalizaci FS eventů (dedupe, merge, overflow signal).
- `src/app/ui/background.rs` má spotřebovat už stabilizovaný batch a provést deterministický handoff do index/reload.
- `src/app/ui/file_tree/mod.rs` zůstává pouze vykonavatel reload handoffu (`request_reload*`), ne místo pro event merge logiku.

### 2) Slabiny, které plán musí zavřít

- Syrový seznam změn bez explicitního merge kontraktu.
- O(event_count) replay i pro redundantní eventy na stejné cestě.
- Chybí first-class overflow větev (safe degradace na single reload místo replay stormu).
- Není zajištěný jednoznačný precedence výsledek při `remove/create/modify` kolizích.

### 3) Minimální cílový tvar řešení

1. Zavést stabilizovaný batch kontrakt z watcher vrstvy.
- Např. `ProjectWatcherBatch { changes: Vec<FsChange>, overflowed: bool }`.

2. Deduplikovat/merge podle cesty s deterministickou precedence.
- Pro stejnou cestu musí být výstup jednoznačný; `Removed` má prioritu.

3. Přidat overflow fallback.
- Při překročení limitu spustit bezpečný full reload path místo granular replay všech eventů.

4. Zachovat non-blocking UI kontrakt.
- Bez blokujících I/O operací v UI vlákně.

## Validation Strategy

Cíl validace: prokázat, že implementace skutečně uzavírá `RELIAB-03` (tj. watcher/event handling po delete/restore nezpůsobuje event storm vedoucí k viditelnému lagu UI) a že změna neporušila existující safety kontrakty.

### Povinné validační vrstvy

1. Unit vrstva (`watcher.rs`)
- Dedupe stejného eventu na stejné cestě.
- Merge kolizí na stejné cestě (hlavně `Modified + Removed` => `Removed`).
- Deterministický výstup při mixu více cest.

2. Overflow vrstva
- Při burstu nad limitem je aktivní `overflowed` kontrakt.
- Overflow větev vede na single fallback reload path (bez granular replay stormu).

3. Orchestrace (`background.rs`)
- Při běžném batchi se aplikují jen deduplikované změny.
- Reload trigger je maximálně jednou za batch a neeskaluje s počtem syrových eventů.

4. Gate vrstva
- `cargo check` PASS.
- `./check.sh` PASS.

### Akceptační výstupy

- Měřitelná evidence pro `RELIAB-03` v artefaktu fáze.
- Traceability: test/ověření -> konkrétní část implementace -> requirement.
- Explicitní záznam, co bylo validováno automaticky a co manuálně.

## Validation Architecture

Tato sekce definuje, jak má být ve fázi plánování vytvořen dokument `38-VALIDATION.md` tak, aby odpovídal Nyquist přístupu (prokázání požadavku přes více nezávislých vrstev, ne jen přes jeden PASS příkaz).

### Požadovaná struktura `38-VALIDATION.md`

1. `# Phase 38 Validation Plan`
- Identifikace fáze, datum, owner, scope guard (`RELIAB-03 only`).

2. `## Requirement Contract`
- Přesná citace/para fráze `RELIAB-03`.
- Negativní definice: co je mimo scope (nové feature, širší refaktory).

3. `## Nyquist Validation Matrix`
- Tabulka minimálně se sloupci:
  - `Signal` (co měříme),
  - `Evidence Type` (unit/contract/manual/gate),
  - `Source` (soubor/test/command),
  - `Pass Criteria`,
  - `Failure Impact`.
- Požadované signály:
  - správnost merge/dedupe,
  - aktivace overflow fallback,
  - absence reload storm patternu v orchestrace větvi,
  - final gate build/check PASS.

4. `## Test Design`
- Seznam konkrétních testů, které musí vzniknout nebo být upraveny.
- U každého testu povinně uvést: cílovou funkci/modul, vstupní scénář, očekávaný výstup, jak mapuje na `RELIAB-03`.

5. `## Manual Verification Scenario`
- Krátký delete->restore burst scénář.
- Jasné PASS/FAIL podmínky (UI lag symptom, reload loop symptom, konzistence strom/tabů).

6. `## Gate Execution Plan`
- Přesné příkazy (`cargo check`, `./check.sh`).
- Definice, kdy je fáze blokovaná (jakýkoliv FAIL).

7. `## Evidence Recording Rules`
- Každý signál v matici musí mít odkaz na konkrétní důkaz (test output, commit diff, run log).
- Pokud něco nejde automatizovat, musí být explicitně označeno jako manuální evidence a proč.

### Nyquist pravidla pro plánovače (povinné)

- Nesmí existovat single-point validation: `cargo check` samotné nikdy nestačí.
- Každý kritický risk musí mít minimálně dvě nezávislé evidence vrstvy (např. unit + orchestration test, nebo test + manuální scénář).
- Validation plán musí mít předem definovaná FAIL kritéria; ne jen seznam aktivit.
- `38-VALIDATION.md` musí vzniknout během plánování před implementací, aby šel plán exekuce řídit proti fixním validačním cílům.

### Definice dokončení validation architektury

`38-VALIDATION.md` je připravené pro exekuci, pokud:
- obsahuje všechny výše uvedené sekce,
- Nyquist matice pokrývá všechny čtyři povinné signály,
- každá plánovaná validační položka má měřitelné PASS/FAIL kritérium,
- existuje přímé mapování `RELIAB-03 -> testy/scénáře/gate`.

## Doporučený tvar plánu (waves)

1. Wave 1: watcher stabilizační jádro (`watcher.rs`) + unit testy merge/overflow.
2. Wave 2: background integrace (`background.rs`) + overflow fallback orchestrace.
3. Wave 3: validace dle `38-VALIDATION.md` + final gate evidence.

## Otevřená rozhodnutí k uzamčení v plánování

- Finální precedence tabulka pro všechny kombinace (`Created/Modified/Removed`) na stejné cestě.
- Přesný overflow práh a přesná fallback akce (`request_reload` vs kombinace s full rescan).
- Finální API shape (`poll()` vs `poll_batch()`), tak aby diff zůstal minimální.

## Out-of-Scope Guard

- Žádné nové trash UI capability.
- Žádný refaktor mimo watcher/background pipeline.
- Žádná změna single-process multi-window architektury.

## RESEARCH COMPLETE
