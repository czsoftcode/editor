# Phase 25: Unsaved Close Guard - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Fáze 25 doručuje ochranu proti ztrátě neuložených změn při zavírání tabu i aplikace/projektu:
- zavření dirty tabu vždy otevře rozhodovací dialog,
- ukončení aplikace/projektu při existenci dirty tabu spustí close flow,
- close flow má konzistentní větve `Save`, `Discard`, `Cancel`,
- při save failu během close flow se zavření nedokončí bez explicitního rozhodnutí uživatele.

Mimo scope této fáze:
- bulk akce typu `Save all` / `Discard all` (future requirement),
- nový capability scope mimo guard flow,
- vizuální polish nad rámec funkčního rozhodovacího flow (patří do následné polish fáze).

</domain>

<decisions>
## Implementation Decisions

### Close flow při více neuložených tabech
- Při ukončení aplikace/projektu se guard kontrola provádí napříč **všemi okny projektu**.
- Pokud je dirty více tabů, flow běží **postupně tab po tabu** (bez bulk akcí).
- Pořadí řešení: **aktivní tab první**, potom zbytek v deterministickém pořadí.
- Dialog pro každý tab musí minimálně ukázat: **název souboru + cesta + stav**.

### Chování rozhodovacího dialogu (`Save` / `Discard` / `Cancel`)
- Výchozí/focusovaná akce je **Cancel** (safe default).
- `Esc` znamená **Cancel**.
- Klik na `X` v dialogu znamená **Cancel**.
- `Enter` spouští aktuálně focusovanou akci.

### Save fail během close flow
- Chyba save se ukáže **inline v tom samém dialogu + toast**.
- Po save failu flow **zůstane na stejném tabu** a vyžádá nové rozhodnutí.
- Po save failu je povolená volba **Discard** i **Cancel** (kromě retry/save).
- Retry save je dostupný přímo ve stejném dialogu (bez opuštění close flow).

### Spouštěče guardu
- Pro zavření tabu guard musí běžet konzistentně pro: **Ctrl+W**, **Menu Close Tab**, **X na tabu**.
- Pro ukončení guard musí běžet konzistentně pro: **window close**, **Quit**, **Close Project**.
- Volba `Cancel` **přeruší celou close operaci**.
- Pokud nejsou neuložené změny, zavření proběhne **okamžitě bez dialogu**.

### Claude's Discretion
- Přesné copy texty dialogu (titulek, krátké vysvětlení, wording chyb).
- Přesné rozložení dialogu a vizuální hierarchie tlačítek při zachování rozhodnutých akcí.
- Konkrétní interní reprezentace close queue (sekvence tabů napříč okny).
- Konkrétní mapování i18n klíčů pro nové guard zprávy.

</decisions>

<specifics>
## Specific Ideas

- Preferovat bezpečné chování bez překvapení: `Cancel` jako default a jednotná semantika pro `Esc`/`X`.
- U více dirty tabů zachovat jednoduchý sekvenční průchod, ne zavádět nové bulk capability.
- Při chybě save držet uživatele v kontextu konkrétního tabu, aby se neztratil důvod selhání.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/workspace/mod.rs`: centrální klávesové zkratky (`Ctrl+S`, `Ctrl+W`) a workspace-level akce.
- `src/app/ui/workspace/menubar/mod.rs`: menu akce `Save`/`Close Tab` a jejich napojení na workspace flow.
- `src/app/ui/editor/files.rs`: `modified` stav, `save`/`save_path`, chování při save failu.
- `src/app/ui/dialogs/confirm.rs`: existující modal pattern pro quit/close potvrzení.
- `src/app/ui/widgets/modal.rs`: jednotný modal framework a confirm/cancel UX patterny.

### Established Patterns
- Rozhodovací modaly mají konzistentní footer akce a cancel větev.
- Save chyby jsou lokalizované a surfacované přes toast/error string.
- Workspace-level orchestrace akcí je preferovaná před rozptýlenou logikou ve více view vrstvách.

### Integration Points
- Tab-close guard vstupy: `Ctrl+W`, menu `Close Tab`, tab close button (v editor/tab render flow).
- App/project close guard vstupy: viewport close request + `QuitAll` + `show_close_project_confirm` flow.
- Save during close: reuse existující save pipeline z editor/workspace modulů.
- Chybové větve a lokalizace: `locales/*/errors.ftl` + guard-specific i18n klíče.

</code_context>

<deferred>
## Deferred Ideas

- `Save all` / `Discard all` bulk akce při app close (future requirement, mimo scope Phase 25).
- Rozšířené UX varianty close flow (např. souhrnný list s bulk volbami) pro budoucí fázi.

</deferred>

---

*Phase: 25-unsaved-close-guard*
*Context gathered: 2026-03-09*
