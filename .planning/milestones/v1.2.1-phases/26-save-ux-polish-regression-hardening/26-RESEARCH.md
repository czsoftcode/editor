# Phase 26 Research: Save UX Polish + Regression Hardening

## Objective
Zodpovědět praktickou otázku: co je potřeba vědět, aby šla fáze 26 dobře naplánovat bez scope creep.

Cíl fáze je úzce vymezen:
- naplnit `MODE-04` (uživatel jasně vidí aktivní save režim),
- dotáhnout UX čitelnost save/dirty signalizace,
- přidat cílené regresní testy pro save a guard větve,
- nepřidat regresi idle výkonu.

## Locked Decisions (From 26-CONTEXT.md)
Následující rozhodnutí jsou závazná pro plán, neotevírat je znovu:
- Aktivní save režim musí být trvale viditelný ve status baru; tab indikace je doplňková.
- Primární text režimu má být explicitní (`Manual Save` / `Auto Save`), ne zkratky.
- Mimo Settings se zobrazuje runtime režim, ne draft.
- Dirty stav má vyšší vizuální prioritu než samotný save režim.
- `Ctrl+S` na již uloženém souboru zachovává info toast.
- Dedupe save chyb zůstává `1.5s`.
- Save fail v guard flow zůstává `inline + toast`.
- `Ctrl+S` bez aktivního tabu zůstává no-op bez toastu.
- Regression hardening je cílený (save/guard/i18n), bez velkého benchmark frameworku.

## Project Constraints Relevant for Planning
- `check.sh` vynucuje `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all-targets --all-features`.
- V `STATE.md` je explicitně uvedeno, že `./check.sh` může padat na repo-wide fmt drift mimo scope; plán musí mít pragmatický gate (`cargo check` + cílené testy) a `./check.sh` evidovat informativně.
- V repu nebyly nalezeny projektové skilly v `.claude/skills/` ani `.agents/skills/`; není třeba plánovat skill-specifickou implementační šablonu.

## Current Baseline (Already Implemented)
1. Viditelnost save režimu už existuje na runtime vrstvě.
- `src/app/ui/workspace/mod.rs`: `save_mode_status_key(...)` + label ve spodním status baru.

2. Save flow je už centralizovaný.
- `handle_manual_save_action(...)` v `workspace/mod.rs` sjednocuje `Ctrl+S` větve (`settings draft`, `save`, `already-saved`, `no-active-tab`).

3. Guard flow (Phase 25) je hotový a testovaný.
- `process_unsaved_close_guard_dialog(...)` řeší `Save/Discard/Cancel`, inline chybu i toast, a neuzavírá při save fail.
- Testy v `src/app/ui/workspace/tests/unsaved_close_guard.rs` kryjí klíčové guard regresní scénáře.

4. Save error dedupe je hotové.
- `src/app/types.rs`: `SAVE_ERROR_DEDUPE_WINDOW = 1500ms` + `should_emit_save_error_toast(...)`.

5. i18n parity check už existuje globálně.
- `src/i18n.rs`: `all_lang_keys_match_english()`.

## Gap Analysis (What Is Still Missing for Phase 26)
1. `MODE-04` je částečně splněný, ale není dotažená UX konzistence.
- Save mode text je ve status baru, ale není pokryt cílenými testy, že zůstává správný při relevantních runtime přechodech.
- Doplňková tab indikace pro režim není explicitně standardizovaná (v tabu je dnes řešen hlavně dirty symbol `●`).

2. Kontrast/čitelnost je implementačně řešená, ale bez přímého regression guardu pro save indikaci.
- Status bar má různé barvy pro dark/light, ale chybí specifický test lock na save-related čitelnost.

3. Save UX větve nejsou kompletně testované jako kontrakt fáze 26.
- Chybí explicitní testy pro kombinaci: `Ctrl+S` + no active tab + already saved + guard save fail UX očekávání.
- Stávající test `status_bar_uses_mode_specific_save_mode_key` ověřuje jen mapování klíčů, ne runtime scénář.

4. Idle výkon má pravidla v runtime, ale fáze 26 potřebuje cílený anti-regression check.
- Repaint throttling logika už existuje; chybí lehký test/assert, že save UX změny nepřidají nové periodické repaint trigger větve.

## Plan-Critical Integration Points
- `src/app/ui/workspace/mod.rs`
: save mode status label, `Ctrl+S` routing, guard save-fail toast/inline flow.
- `src/app/ui/editor/ui.rs`
: save status (`Unsaved/Saving/Saved`) rendering v editor status baru, light/dark barvy.
- `src/app/ui/editor/render/tabs.rs`
: tab label signalizace (`●` pro modified), místo pro doplňkovou nenásilnou indikaci režimu (pokud bude potřeba).
- `src/app/types.rs`
: save error dedupe kontrakt (`1.5s`) musí zůstat beze změny.
- `locales/*/ui.ftl`, `locales/*/errors.ftl`
: klíče pro mode/status/save/guard texty; změny musí projít přes 5 jazyků.

## Recommended Plan Shape
1. Plan 26-01: Save Mode Visibility Contract
- Zafixovat finální UX pravidla (status bar primární, tab sekundární) a doplnit minimální UI patch jen tam, kde je skutečná mezera.
- Přidat unit/integration-lite testy pro runtime zobrazení aktivního režimu.

2. Plan 26-02: Dirty vs Save Mode Visual Priority
- Ujistit se, že při současném zobrazení je dirty stav vizuálně dominantní.
- Přidat regression testy na light/dark čitelnost relevantních save/dirty indikátorů (bez redesignu UI).

3. Plan 26-03: Save Feedback Regression Pack
- Dopsat cílené testy pro `Ctrl+S` větve: modified, already-saved, no-active-tab.
- Dopsat testy pro guard save-fail (`inline + toast`, bez zavření).
- Ověřit, že dedupe okno 1.5s zůstává zachováno.

4. Plan 26-04: Idle Safety Guard + i18n Smoke
- Přidat lehký guard proti regresím repaint/idle chování v save UX cestách.
- Doplnit i18n smoke pro save mode/status klíče ve všech jazycích (nad rámec obecné parity).

## Risks and Mitigations
- Riziko: přidání dalšího „živého“ repaint triggeru kvůli UX polish.
- Mitigace: držet změny čistě event-driven; bez nových periodických timer větví.

- Riziko: testy budou validovat jen klíče, ne skutečné chování.
- Mitigace: kombinovat mapovací unit test + behavior testy nad `manual_save_request` a guard reducer logikou.

- Riziko: i18n regressions při přidání/úpravě textů.
- Mitigace: všechny nové klíče přidat napříč 5 jazyky a v gate vždy spustit `all_lang_keys_match_english`.

- Riziko: scope creep do nových save capability.
- Mitigace: držet se výhradně `MODE-04` + regression hardening, bez nových workflow funkcí.

## Validation Architecture
Tato sekce je připravená jako podklad pro `26-VALIDATION.md`.

### Requirement Coverage Contract
- `MODE-04` musí být pokryto kombinací:
  - automated: unit/integration-lite testy save mode indikace + save branch behavior,
  - manual: rychlý UX smoke v light/dark pro jasnost režimu a prioritu dirty stavu.

### Verification Layers
1. Unit layer (rychlá zpětná vazba)
- `manual_save_request(...)` větve:
  - settings modal open -> `SaveSettingsDraft`,
  - active modified -> `SaveEditorFile`,
  - active clean -> `ShowAlreadySavedInfo`,
  - no active tab -> `NoActiveTab`.
- `save_mode_status_key(...)` mapování zůstává správné.
- dedupe kontrakt: `SAVE_ERROR_DEDUPE_WINDOW` + `save_error_dedupe_decision`.

2. Integration-lite layer (workflow jistota)
- Guard save fail drží flow aktivní, nastaví inline chybu a negeneruje close side-effect.
- `Ctrl+S` routing v workspace vrstvě nedegraduje guard ani settings behavior.

3. i18n layer
- `all_lang_keys_match_english` jako povinný gate.
- Přidat save UX smoke assert na přítomnost klíčů používaných v této fázi (`statusbar-save-mode-*`, `statusbar-unsaved/saved`, `unsaved_close_guard_save_failed`).

4. Manual UAT layer (krátký, cílený)
- Scenario A: Light mode, modified tab, jasně viditelný active save mode a zároveň dominantní dirty signál.
- Scenario B: Dark mode, stejný scénář.
- Scenario C: `Ctrl+S` na clean tab -> info toast.
- Scenario D: `Ctrl+S` bez aktivního tabu -> no-op bez toastu.
- Scenario E: save fail v guard dialogu -> inline + toast, zavírání se nedokončí.

### Suggested Gates During Execution
- Po každém tasku: `cargo check`.
- Po každé wave: cílené testy pro save/guard/i18n (bez čekání na plný suite).
- Před final verify: `cargo check` + `./check.sh` (s poznámkou, že repo-wide fmt drift může být mimo scope fáze).

### Evidence Map Template (for 26-VALIDATION)
- `MODE-04`:
  - automated evidence: názvy konkrétních testů v `workspace/mod.rs`, `workspace/tests/unsaved_close_guard.rs`, případně `editor/ui.rs`.
  - manual evidence: výsledky scénářů A-E (PASS/FAIL).
- Idle safety:
  - automated evidence: test/assert že save UX úpravy nepřidaly novou periodickou repaint větev.

## Planning Checklist (What You Need to Know)
- Implementační základ save/guard už existuje; fáze 26 je hlavně polish + test hardening, ne nový subsystém.
- Nejdůležitější plánovací rozhodnutí je vybrat minimální UI zásah pro MODE-04 bez vizuálního šumu.
- Největší technická hodnota fáze je regression pack (save branches + guard fail + i18n + idle safety).
- Gate strategie musí respektovat realitu repa (`cargo check` povinně, `./check.sh` informativně pokud padá mimo scope).

## RESEARCH COMPLETE
Fáze 26 má nízké architektonické riziko a vysoký testovací dopad. Pro dobrý plán je klíčové držet scope na `MODE-04`, udělat jen minimální UI polish, a většinu energie dát do cílené regression coverage save/guard větví a lehkého idle safety guardu.
