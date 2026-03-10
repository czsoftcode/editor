# Phase 26: Save UX Polish + Regression Hardening - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Fáze 26 řeší UX vyjasnění už existujícího save flow a regresní hardening.
Scope je omezený na:
- jasnou a konzistentní viditelnost aktivního save režimu,
- čitelnost dirty/clean indikace v light i dark mode,
- konzistentní feedback ve save větvích,
- regresní testy pro save/guard větve a lehkou kontrolu bez dopadu na idle výkon.

Mimo scope:
- nové capability (např. nové save workflow funkce, bulk akce, command palette redesign),
- velký performance benchmark framework.

</domain>

<decisions>
## Implementation Decisions

### Viditelnost aktivního save režimu (MODE-04)
- Aktivní save režim bude trvale viditelný ve **status baru + doplňkově v tab indikaci**.
- Primární label bude **explicitní text** (`Manual Save` / `Auto Save`), ne zkratky.
- Mimo Settings se vždy zobrazuje **jen aktivní runtime režim** (draft režim se globálně nepromítá před Save).
- Pro manual režim se nepřidává extra varovná vrstva nad existující dirty signalizaci.

### Dirty/Clean čitelnost a kontrast
- V tabu zůstane jednoduchý symbol (`●`), detail stavu bude čitelný ve status baru.
- V light mode se použije strategie **posílení stávajícího kontrastu** (bez kompletní změny vizuálního jazyka).
- Pokud je současně vidět save režim i dirty stav, **dirty stav má vyšší vizuální prioritu**.
- Součástí fáze budou **regression testy čitelnosti/konzistence indikace** pro light/dark.

### Save feedback a chybové větve
- `Ctrl+S` na již uloženém souboru: zachovat **krátký info toast**.
- Dedup opakovaných save chyb: ponechat **1.5s** okno.
- Save fail při close guard flow: zachovat **inline chyba + toast**.
- `Ctrl+S` bez aktivního tabu: ponechat **bez akce a bez toastu**.

### Regression hardening rozsah
- Zaměřit se na **cílené save/guard regresní testy** (bez rozšíření do mimo-scope oblastí).
- Přidat **lehký guard check** proti regresím idle výkonu (bez plného benchmark projektu).
- i18n smoke/regression coverage pro save UX bude pro **všech 5 jazyků**.
- Gate pro fázi: `cargo check` + cílené testy; `./check.sh` běh informativně kvůli známému repo-wide fmt driftu.

### Claude's Discretion
- Přesné umístění a vizuální hierarchie doplňkové tab indikace tak, aby nenavyšovala vizuální šum.
- Konkrétní barevné hodnoty pro light/dark kontrast v mezích existujícího design jazyka.
- Konkrétní podoba lehkého idle guard checku (metoda/heuristika) bez přidávání nových heavy závislostí.

</decisions>

<specifics>
## Specific Ideas

- Důraz na okamžitou srozumitelnost: uživatel má na první pohled vědět, v jakém save režimu je, ale bez přetížení UI.
- Dirty stav je rizikovější signál než samotný režim, proto má mít vyšší prioritu.
- Save feedback má zůstat konzistentní s dosavadním chováním (včetně dedup pravidel).
- Regression hardening má být praktický a rychlý, ne rozšířený do nové výkonnostní platformy.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/workspace/mod.rs`: už obsahuje save mode status key mapping, `Ctrl+S` routing, guard orchestrace a toast handling.
- `src/app/ui/editor/ui.rs`: status bar rendering a existující text/symbol signalizace save stavu.
- `src/app/ui/workspace/modal_dialogs/settings.rs`: save mode radio volba, mode-change toast a draft/apply lifecycle.
- `src/app/ui/workspace/tests/unsaved_close_guard.rs`: existující guard regresní testy jako základ pro rozšíření coverage.
- `locales/*/ui.ftl` + `locales/*/errors.ftl`: existující i18n klíče pro save režim, dirty stav a guard save fail větev.

### Established Patterns
- Settings změny se promítají až po explicitním `Save` (draft lifecycle).
- Save/guard větve jsou centralizované ve workspace vrstvě.
- Chybové save větve používají toast + lokalizované message klíče.
- Testy jsou inline (`#[cfg(test)]`) v dotčených modulech.

### Integration Points
- Save mode indikace: `workspace/mod.rs` + `editor/ui.rs` (status/tab vrstva).
- Kontrast a čitelnost dirty/clean: status/tab rendering v editor UI + příslušné i18n labely.
- Save feedback policy: `handle_manual_save_action`, guard save fail branch, toast dedup policy.
- Regression hardening: rozšíření existujících test modulů pro save/guard/i18n scénáře.

</code_context>

<deferred>
## Deferred Ideas

- Žádné nové deferred capability nevznikly; diskuze zůstala v rámci scope Phase 26.

</deferred>

---

*Phase: 26-save-ux-polish-regression-hardening*
*Context gathered: 2026-03-10*
