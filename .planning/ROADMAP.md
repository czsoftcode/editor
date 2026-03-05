# Roadmap: PolyCredo Editor — Dark/Light Mode

## Overview

Tento milestone implementuje plnohodnotný light mode pro PolyCredo Editor. Práce je rozdělená do čtyř fází podle závislostí: nejprve egui Visuals a syntax highlighting jako základ (bez něj nic nefunguje), pak terminál a git barvy, následně tři varianty světlé palety v Settings modalu a nakonec infrastrukturní gap-closure kolem sandbox nastavení. Výsledkem je editor, kde uživatel přepne téma a celá aplikace — editor, terminál, file tree, dialogy — odpovídá zvolenému režimu bez vizuálních defektů.

## Phases

- [x] **Phase 1: Základ** - egui Visuals + Highlighter — funkční dark/light přepínač s čitelným syntax highlighting (completed 2026-03-04)
- [x] **Phase 2: Terminal + Git barvy** - Terminál a file tree použitelné v light mode (completed 2026-03-04)
- [x] **Phase 3: Light varianty + Settings UI** - Tři varianty světlé palety, picker v Settings, kompletní polish (completed 2026-03-05)
- [x] **Phase 4: Infrastructure** - Sandbox settings UX + infrastrukturní gap closure (completed 2026-03-05)

## Phase Details

### Phase 1: Základ
**Goal**: Uživatel může přepnout dark/light mode a celý editor (UI panely, menu, dialogy, syntax highlighting) odpovídá zvolenému režimu bez flash při startu
**Depends on**: Nothing (first phase)
**Requirements**: THEME-01, THEME-02, THEME-03, THEME-04, EDIT-01, EDIT-02, EDIT-03, EDIT-04, SETT-04, UI-01, UI-02, UI-03
**Success Criteria** (what must be TRUE):
  1. Uživatel přepne na light mode a editor se okamžitě zobrazí světlý — menu, panely, dialogy bez tmavých artefaktů
  2. Syntax highlighting v light mode je čitelný — žlutá barva nesplývá s bílým pozadím (Solarized Light aktivní)
  3. Při startu aplikace s uloženým light mode není žádný tmavý záblesk — téma se aplikuje v `new()`, ne `update()`
  4. Načtení starého `settings.json` bez `light_variant` pole neskončí pádem aplikace
  5. Záložky editoru a status bar jsou čitelné v obou módech — indikátor neuloženého stavu (●) viditelný
**Plans:**
- [x] 01-01-PLAN.md — Settings struct rozšíření (LightVariant, syntect_theme_name, to_egui_visuals)
- [x] 01-02-PLAN.md — Highlighter parametrizace + startup apply (theme_name parametr, cache invalidace)
- [x] 01-03-PLAN.md — Theme-aware floating frame fill pro StandardTerminalWindow (bez hardcoded tmavé výplně)
- [x] 01-04-PLAN.md — Theme-aware status bar text kontrast pro light mode (UI-03 gap closure)

### Phase 2: Terminal + Git barvy
**Goal**: Terminál (Claude panel i Build terminál) a soubory ve file tree jsou čitelné v light mode — žádné tmavé terminály na světlém pozadí, žádné neviditelné git statusy
**Depends on**: Phase 1
**Requirements**: TERM-01, TERM-02, TERM-03, TERM-04, TREE-01, TREE-02
**Success Criteria** (what must be TRUE):
  1. Claude panel v light mode zobrazuje světlé pozadí — text je čitelný, ne bílý na černém
  2. Build terminál v light mode zobrazuje světlé pozadí — konzistentní s editorem
  3. Soubory s git statusem M/A/?? jsou čitelně obarveny na světlém pozadí — zejména untracked (šedá) není neviditelná
  4. Scrollbar terminálu není hardcoded tmavý — barva odpovídá aktivnímu tématu
**Plans:**
- [x] 02-01-PLAN.md — Theme-aware terminál rendering + runtime přepínání tématu + scrollbar z `ui.visuals()`
- [x] 02-02-PLAN.md — Semantické git statusy + explicitní light/dark paleta pro file tree

### Phase 3: Light varianty + Settings UI
**Goal**: Uživatel si vybere ze tří variant světlé palety v Settings panelu a změna se okamžitě projeví v celé aplikaci bez restartu
**Depends on**: Phase 1, Phase 2
**Requirements**: LITE-01, LITE-02, LITE-03, LITE-04, SETT-01, SETT-02, SETT-03
**Success Criteria** (what must be TRUE):
  1. Settings panel nabízí výběr ze tří světlých variant (Teplá slonová kost, Studená šedá, Sépiová) — picker je viditelný pouze v light mode
  2. Kliknutí na variantu okamžitě změní pozadí celé aplikace — žádný restart nutný (live preview)
  3. Zvolená varianta přežije restart aplikace — načte se z canonical `settings.toml` (`settings.json` zůstává jen legacy vstup pro migraci)
  4. Panely jsou vizuálně odlišeny pomocí `faint_bg_color` — editor, file tree a side panel nejsou splývavé
**Plans:** 5/5 plans complete
- [x] 03-01-PLAN.md — LightVariant enum + Settings rozšíření + to_egui_visuals mapping
- [x] 03-02-PLAN.md — Settings UI picker variant + visibility only in light mode
- [x] 03-03-PLAN.md — Live preview variant bez restartu
- [x] 03-04-PLAN.md — Persist varianty v canonical `settings.toml` + finalize polish
- [x] 03-05-PLAN.md — Gap closure: oprava expanze pickeru karet + teplý tón terminálu WarmIvory

### Phase 4: Infrastructure
**Goal**: Dokoncit infrastrukturu sandbox nastaveni a odstranit UAT gapy v discoverability sandbox tooltipu
**Depends on**: Phase 3
**Requirements**: SETT-01, SETT-02, SETT-03, SETT-04, SETT-05, TERM-01, TERM-02, TERM-03
**Success Criteria** (what must be TRUE):
  1. Sandbox rezim je persistovany v settings.toml a aplikuje se konzistentne po reopen projektu
  2. Terminály a build flow respektuji sandbox vs project root rezim
  3. Tooltip u sandbox prepinace je snadno objevitelny a inline poznamka o reopen je citelna
**Plans:** 2/2 plans complete
- [x] 04-01-PLAN.md — Persist sandbox mode + workspace/terminal wiring
- [x] 04-02-PLAN.md — UX gap closure: tooltip discoverability + inline note emphasis

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Základ | 4/4 | Complete | 2026-03-04 |
| 2. Terminal + Git barvy | 2/2 | Complete   | 2026-03-04 |
| 3. Light varianty + Settings UI | 5/5 | Complete   | 2026-03-05 |
| 4. Infrastructure | 2/2 | Complete | 2026-03-05 |

### Phase 5: Okamžité aplikování změny režimu sandboxu po přepnutí checkboxu

**Goal:** Po Save v Settings se sandbox režim aplikuje okamžitě bez reopen, bezpečně pro terminály, file tree, otevřené taby a staged/sync flow
**Requirements**: SANDBOX-01, SANDBOX-02, SANDBOX-03, SANDBOX-04
**Depends on:** Phase 4
**Plans:** 3/3 plans complete

Plans:
- [ ] 05-01-PLAN.md — Save/Cancel flow + runtime apply helper + multi-window dispatch
- [ ] 05-02-PLAN.md — Terminály + file tree + přemapování tabů
- [ ] 05-03-PLAN.md — Staged blokace OFF + sync při ON
