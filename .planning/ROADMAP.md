# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- ✅ **v1.1.0 Sandbox Removal** — Phases 9-12 (shipped 2026-03-06)
- ✅ **v1.2.0 AI Chat Rewrite** — Phases 13-18 (shipped 2026-03-06)
- ⏸️ **v1.2.1-dev GSD Integration + Slash Commands** — Phases 19-23 (deferred)
- [ ] **v1.3.0 Save Modes + Unsaved Changes Guard** — Phases 24-26 (in progress)

## Phases

### v1.3.0 Save Modes + Unsaved Changes Guard (In Progress)

**Milestone Goal:** Zpřehlednit a zbezpečnit ukládání v editoru přes Ctrl+S default, přepínání auto/manual režimu a ochranu proti ztrátě neuložené práce při zavírání tabu nebo aplikace.

- [ ] **Phase 24: Save Mode Foundation** — Ctrl+S default, save mode settings persistence, runtime apply
- [ ] **Phase 25: Unsaved Close Guard** — tab close + app close confirm flow, save/discard/cancel decision handling
- [ ] **Phase 26: Save UX Polish + Regression Hardening** — mode/status UI clarity, edge-case coverage, regression tests

## Phase Details

### Phase 24: Save Mode Foundation
**Goal**: Uživatel má předvídatelné ukládání s Ctrl+S default a přepínatelným auto/manual režimem.
**Depends on**: Phase 23 (latest completed system baseline)
**Requirements**: SAVE-01, SAVE-02, SAVE-03, MODE-01, MODE-02, MODE-03
**Success Criteria** (what must be TRUE):
1. `Ctrl+S` uloží aktivní tab a přepne stav souboru na uložený bez nutnosti měnit fokus.
2. V Settings existuje přepínač `Automatic Save` / `Manual Save` a jeho volba se persistuje přes restart.
3. Po uložení Settings se save režim aplikuje okamžitě bez restartu aplikace.
4. Chyby ukládání se propisují do UI (toast/message), nejsou potichu ignorované.
**Plans**: TBD

### Phase 25: Unsaved Close Guard
**Goal**: Zabránit ztrátě neuložených změn při zavírání tabu i aplikace.
**Depends on**: Phase 24
**Requirements**: GUARD-01, GUARD-02, GUARD-03, GUARD-04
**Success Criteria** (what must be TRUE):
1. Zavření tabu s neuloženými změnami vždy vyvolá rozhodovací dialog.
2. Zavření aplikace s libovolným neuloženým tabem vždy vyvolá rozhodovací dialog.
3. Dialog umožní `Save`, `Discard`, `Cancel` a větve se chovají konzistentně.
4. Pokud save během close flow selže, uživatel je informován a close se nedokončí bez rozhodnutí.
**Plans**: TBD

### Phase 26: Save UX Polish + Regression Hardening
**Goal**: Uživatel jasně vidí aktivní save režim a save flow je pokrytý regresními testy.
**Depends on**: Phase 25
**Requirements**: MODE-04
**Success Criteria** (what must be TRUE):
1. UI konzistentně zobrazuje aktivní save režim na relevantních místech.
2. Dirty/clean indikace je čitelná v light i dark mode.
3. Existují regresní testy pro Ctrl+S, close guard (tab/app) a save failure větve.
4. Nejsou regresní dopady na idle výkon editoru.
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 24. Save Mode Foundation | 2/4 | In Progress|  | - |
| 25. Unsaved Close Guard | v1.3.0 | 0/? | Not started | - |
| 26. Save UX Polish + Regression Hardening | v1.3.0 | 0/? | Not started | - |

## Known Issues / TODO

- Legacy quick tasks table ve `STATE.md` obsahuje historické řádky mimo standard tabulku; při dalším housekeepingu sjednotit formát.
- `./check.sh` aktuálně padá na repo-wide `cargo fmt --check` rozdílech mimo scope této roadmap inicializace.
