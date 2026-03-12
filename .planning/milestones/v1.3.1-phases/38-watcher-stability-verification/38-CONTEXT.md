# Phase 38: Watcher Stability + Verification - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Stabilizovat watcher eventy po delete/restore workflow tak, aby nevznikal event storm vedouci k viditelnemu lagovani UI, a uzavrit fazi auditovatelnou verifikaci (cargo check + ./check.sh).

Scope je omezen na RELIAB-03 v ramci existujiciho trash/restore toku (bez novych user feature capability mimo watcher stabilitu).

</domain>

<decisions>
## Implementation Decisions

### Event pipeline policy (watcher)
- Deduplikace v burstu bude po klici path + kind (nikoli pouze path).
- Batching okno bude 100-150 ms.
- Pri kolizi eventu na stejne ceste ma remove prioritu nad modify/create.
- Pri overflow nebo event spicce se pouzije bezpecny fallback: jeden full reload misto detailniho replaye vsech eventu.

### Carried Forward z predchozich fazi
- .polycredo namespace zustava interni a ma byt i nadale watcher-ignore.
- Delete/restore tok zustava fail-closed a I/O bezi mimo UI vlakno.
- Restore UI kontrakt z faze 37 zustava: reload + highlight bez auto-open tabu.

### Claude's Discretion
- Presna podoba interni datove struktury pro batch queue (Vec/HashMap/ordered merge).
- Presne prahy, kdy pipeline prepne z detailniho zpracovani na overflow fallback.
- Detail wordingu pripadneho info signalu pri fallback reloadu (pokud bude potreba uzivatelsky surfacovat).
- Konkretni test harness a simulace burst scenaru (pri zachovani RELIAB-03 hranice).

</decisions>

<specifics>
## Specific Ideas

- U burstu je preferovana deterministicka priorita koncoveho stavu (remove), ne slepe prehrani vsech mezikroku.
- Reakce UI ma zustat svizna; proto je explicitne schvalen fallback full reload pri spicce misto blokovani UI.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- src/watcher.rs: FileWatcher::try_recv() a ProjectWatcher::poll() uz vraci davky eventu (aktualne cap 500).
- src/app/ui/background.rs: centralni spotrebitel watcher eventu (process_background_events) a misto pro rizeni reload strategie.
- src/app/ui/file_tree/mod.rs: request_reload() a request_reload_and_expand() pro bezpecny synchronizacni fallback.

### Established Patterns
- Event processing je polling-based pres try_recv() + per-frame zpracovani.
- .polycredo a high-frequency adresare (.git, target, node_modules, ...) jsou ve watcheru uz filtrovany.
- Toast-first error surfacing pipeline je uz zavedena ve workspace/file-tree toku.

### Integration Points
- src/watcher.rs: implementace dedupe/batching/merge pravidel a overflow fallback triggeru.
- src/app/ui/background.rs: aplikace batch vystupu do project_index, reload triggeru a tab synchronizace.
- src/app/ui/file_tree/mod.rs: finalni reload/expand handoff pri stabilizacnich scenarich.

</code_context>

<deferred>
## Deferred Ideas

- Zadne nove capability nebyly pridany; diskuse zustala v hranici faze 38.

</deferred>

---

*Phase: 38-watcher-stability-verification*
*Context gathered: 2026-03-12*
