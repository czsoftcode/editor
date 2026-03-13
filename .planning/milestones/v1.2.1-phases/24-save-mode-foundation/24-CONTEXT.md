# Phase 24: Save Mode Foundation - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Fáze 24 doručuje základ ukládání: `Ctrl+S` jako výchozí ruční uložení aktivního tabu, přepínání `Automatic Save` / `Manual Save` v Settings, persistenci režimu a okamžitý runtime apply po `Save`. Součástí je i robustní save feedback/error policy.

Mimo scope této fáze:
- potvrzovací flow při zavření tabu/aplikace s neuloženými změnami (Phase 25)
- větší UX polish vrstvy (Phase 26)
- nový readonly režim editoru (deferred)

</domain>

<decisions>
## Implementation Decisions

### Ctrl+S behavior
- `Ctrl+S` ukládá pouze **aktivní tab**.
- Když je otevřený modal (např. Settings), shortcut respektuje modal kontext (neprovádí file-save v pozadí).
- V rámci modalu Settings je `Ctrl+S` interpretován jako save settings draftu.
- Menu Save a `Ctrl+S` sdílí stejný handler (100% konzistentní behavior).
- Při konfliktu externí změny má prioritu konflikt flow (ne přímý overwrite).
- Pro nezměněný soubor se zobrazuje info feedback "Soubor už je uložen".
- U binárních tabů je požadavek na "Save As only" odložen mimo tuto fázi.

### Save mode rules (Auto/Manual)
- Výchozí režim po upgradu je **Manual**.
- Režim je **globální pro celou aplikaci** (ne per-tab/per-project).
- `Automatic Save` zachovává existující debounce trigger (`try_autosave` flow).
- `Manual Save` vypíná background autosave úplně.
- Změna režimu se aplikuje po kliknutí `Save` v Settings (respektuje draft pattern).
- Přepnutí režimu neprovádí speciální zásah do aktuálně dirty tabu; ovlivňuje další save trigger chování.
- Po úspěšné změně režimu se zobrazí krátký info toast.
- Přepínač bude jednoduché radio `Automatic Save` / `Manual Save`.

### Save feedback and error policy
- Save failure používá **error toast** (ne modal) s názvem souboru + důvodem chyby.
- Po save failu zůstává tab ve stavu `Modified`.
- Auto-save retry po failu běží až po nové editaci (žádný loop spam).
- Opakované stejné chyby mají být deduplikované v krátkém intervalu.
- Info toast "Soubor už je uložen" má stejné pravidlo v Auto i Manual režimu.

### UX visibility in this phase
- Už ve fázi 24 bude minimálně základní runtime indikace aktivního režimu (např. status bar text).
- Detailnější vizuální polish zůstává pro Phase 26.

### Claude's Discretion
- Přesné časové okno deduplikace toastů.
- Finální wording i18n klíčů pro info/error save zprávy.
- Přesná podoba základní mode indikace ve status baru.
- Interní mechanika, jak v runtime gate-nout `try_autosave` podle save režimu.

</decisions>

<specifics>
## Specific Ideas

- Uživatel explicitně chce `Ctrl+S` jako defaultní workflow a jasné oddělení Auto vs Manual režimu.
- Uživatel chce viditelnou zpětnou vazbu i pro případ, kdy soubor už byl uložen.
- Uživatel preferuje konzistenci: stejné save chování napříč shortcutem a menu.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/workspace/mod.rs`: globální `Ctrl+S` shortcut už existuje.
- `src/app/ui/editor/files.rs`: `save`, `save_path`, `try_autosave`, `SaveStatus` flow už pokrývá základní save stavy.
- `src/app/ui/background.rs`: autosave je centralizovaný v background event loopu přes `try_autosave`.
- `src/settings.rs` + `Settings` draft v `modal_dialogs/settings.rs`: existující persist + apply pattern pro nové nastavení.
- `Toast` systém: existující standardní cesta pro info/error feedback.

### Established Patterns
- Settings změny se potvrzují přes `Save/Cancel` draft lifecycle (ne live persist bez potvrzení).
- Runtime apply se propaguje přes `AppShared.settings` + `settings_version`.
- Save chyby se mapují do lokalizovaných chybových zpráv (`i18n.get_args`).

### Integration Points
- Save mode field: `src/settings.rs` (+ default + serde persist).
- Settings UI toggle: `src/app/ui/workspace/modal_dialogs/settings.rs`.
- Autosave gate: `src/app/ui/background.rs` (volání `try_autosave`).
- Manual save path: `src/app/ui/workspace/mod.rs` (`Ctrl+S`) + editor save handler.
- Mode/status indikace: `src/app/ui/editor/ui.rs` status bar rendering.

</code_context>

<deferred>
## Deferred Ideas

- Read-only mód editoru (soubor jen pro čtení, writable default) — nový capability scope mimo Phase 24.
- Binární taby „Save As only“ behavior — navázat na read-only/save workflow rozšíření.

</deferred>

---

*Phase: 24-save-mode-foundation*
*Context gathered: 2026-03-09*
