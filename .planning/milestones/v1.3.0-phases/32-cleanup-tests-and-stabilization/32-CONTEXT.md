# Phase 32: Cleanup, Tests, and Stabilization - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Dokoncit cleanup, testy a dokumentaci po odstraneni CLI vrstvy tak, aby stabilizacni faze uzavirala STAB-01/STAB-02 bez pridavani novych capability.
Scope je stabilizace existujiciho assistant-only AI terminal workflow a konzistence aktivnich v1.3 planning artefaktu.

</domain>

<decisions>
## Implementation Decisions

### Regresni priority (STAB-01/STAB-02)
- Povinne regression-guarded toky: prompt + stream + slash + approval.
- Preferovany test mix: targeted integracni testy + smoke testy.
- Failure/recovery overovat minimalne v kritickych scenarich (toast, retry, approval blokace).
- Finalni gate pro fazi: `cargo check` + `./check.sh` + cilene testy relevantni pro stabilizovane toky.

### Cleanup hranice
- Hard cleanup delat v `src/`, aktivnich `tests/` a aktivnich planning artefaktech milestone v1.3.
- Historicke/archivni artefakty neprepisovat; zachovat auditni stopu.
- Gap closure artefakty z phase 31 brat jako uzavrene; neotvirat novy gap cyklus bez explicitniho duvodu.
- Pri nalezu drobnych nesouladu mimo STAB-01/02 je dovoleny low-risk fix; jinak zapis do deferred.

### Dokumentacni finish
- Povinne artefakty pro sign-off: `ROADMAP.md`, `STATE.md`, `REQUIREMENTS.md`, `32-VERIFICATION.md`.
- Pridat strucny stabilizacni zapis do `CHANGELOG.md`.
- Verifikacni report vest stylem evidence-first (konkretni PASS/FAIL dukazy, bez zbytecneho narativu).
- Nove mimo-scope napady zachytit do Deferred Ideas, neimplementovat v teto fazi.

### Claude's Discretion
- Presny vyber cilenych test case uvnitr zvoleneho mixu (integracni/smoke) podle realneho rizika.
- Poradi cleanup kroku mezi kodem, testy a dokumentaci.
- Minimalni rozsah low-risk textovych oprav v aktivnich planning artefaktech.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/terminal/ai_chat/logic.rs`, `slash.rs`, `approval.rs`: hlavni flow body pro prompt/stream/slash/approval regression coverage.
- `src/app/ui/background.rs`: centralni runtime polling a error/stream processing body pro stabilizacni testy.
- `tests/phase30_plan*.rs`: existujici patterny na grep/audit/regression validace planning claims.

### Established Patterns
- Kvalitativni gate projektu je standardizovany pres `cargo check` a `./check.sh`.
- Assistant-only boundary je uzamcena po phase 31; scope phase 32 ma stabilizovat, ne menit produktove chovani.
- Planning traceability se opira o `REQUIREMENTS.md` + `ROADMAP.md` + phase verification artefakty.

### Integration Points
- Cleanup/stabilizace zasahne hlavne aktivni `src/app/ui/*`, `src/app/ai_core/*`, `tests/*` relevantni pro STAB.
- Dokumentacni uzavreni se napoji na `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/REQUIREMENTS.md` a phase-32 verification.
- Release komunikace: `CHANGELOG.md` (strucny stabilizacni zapis).

</code_context>

<specifics>
## Specific Ideas

- Stabilizace ma byt cilena: uzavrit kvalitu po migraci, ne otevirat dalsi produktovy scope.
- Priorita je zachovat assistant-only AI terminal chovani bez regresi v kritickych runtime tocich.
- Verifikace ma byt dukazova a rychle auditovatelna.

</specifics>

<deferred>
## Deferred Ideas

- Hlubsi historicky cleanup starsich milestone/planning artefaktu mimo aktivni v1.3 scope.
- Sirsi UX polish znameho tech debt mimo explicitni STAB-01/STAB-02.

</deferred>

---

*Phase: 32-cleanup-tests-and-stabilization*
*Context gathered: 2026-03-11*
